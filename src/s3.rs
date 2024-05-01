use anyhow::Context;
use bytes::{Bytes, BytesMut};
use futures::TryStreamExt;

use rusoto_core::request::HttpClient;
use rusoto_core::{Client, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{S3Client, S3};
use std::path::PathBuf;
use std::str::FromStr;

use futures::stream::Stream;
use tokio::io::AsyncRead;
use tokio_util::codec;
use tracing::*;

use serde::Deserialize;

#[derive(Clone)]
pub struct S3Object {
    pub key: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub size: i64,
}

impl std::fmt::Debug for S3Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Object")
            .field("key", &self.key)
            .field("last_modified", &self.last_modified)
            .field("size", &self.size)
            .finish()
    }
}

#[derive(Debug, Deserialize)]
pub enum S3Region {
    /// AWS region
    #[serde(rename = "region")]
    Region(String),
    /// for non AWS buckets, name of the endpoint
    #[serde(rename = "endpoint")]
    Endpoint(String),
}

/// Bucket embeds S3 client object and bucket name
#[derive(Clone)]
pub struct Bucket {
    pub client: S3Client,
    pub bucket: String,
}
impl std::fmt::Debug for Bucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bucket")
            .field("bucket", &self.bucket)
            .finish()
    }
}

fn into_bytes_stream<R>(r: R) -> impl Stream<Item = tokio::io::Result<Bytes>>
where
    R: AsyncRead,
{
    codec::FramedRead::new(r, codec::BytesCodec::new()).map_ok(|bytes| bytes.freeze())
}

impl Bucket {
    /// creates new s3 bucket object
    pub fn new(
        s3_access_key: &str,
        s3_secret_access_key: &str,
        s3_bucket: &str,
        s3_region: &S3Region,
    ) -> anyhow::Result<Self> {
        tracing::debug!("accessing s3 bucket {} in {:?}", s3_bucket, s3_region);

        let region = match s3_region {
            S3Region::Region(r) => Region::from_str(r).context("invalid s3 region")?,
            S3Region::Endpoint(url) => Region::Custom {
                name: "custom".to_string(),
                endpoint: url.to_string(),
            },
        };
        let aws_provider: StaticProvider = StaticProvider::new(
            s3_access_key.to_string(),
            s3_secret_access_key.to_string(),
            None,
            None,
        );
        let http_client =
            match HttpClient::new().with_context(|| format!("Failed to create AWS HTTP client")) {
                Err(e) => return Err(e),
                Ok(x) => x,
            };
        let client = Client::new_with(aws_provider, http_client);
        return Ok(Self {
            client: S3Client::new_with_client(client, region),
            bucket: s3_bucket.to_string(),
        });
    }

    #[instrument(ret, level = "info")]
    pub async fn put_string(&self, filename: &str, contents: String) -> anyhow::Result<u64> {
        let length = contents.len() as u64;
        if length == 0 {
            return Ok(0);
        }
        let put_req = rusoto_s3::PutObjectRequest {
            bucket: self.bucket.clone(),
            key: filename.to_string(),
            content_length: Some(length as i64),
            body: Some(contents.into_bytes().into()),
            ..Default::default()
        };
        let _ = self
            .client
            .put_object(put_req)
            .await
            .context("failed to put object")?;
        Ok(length)
    }

    #[instrument(ret, level = "info")]
    pub async fn put_file(&self, filename: &str, local_filename: &PathBuf) -> anyhow::Result<u64> {
        let tokio_file = tokio::fs::File::open(local_filename)
            .await
            .context("failed to open local file")?;
        let file_size = tokio_file.metadata().await?.len();
        if file_size == 0 {
            return Ok(0);
        }
        let put_req = rusoto_s3::PutObjectRequest {
            bucket: self.bucket.clone(),
            key: filename.to_string(),
            content_length: Some(file_size as i64),
            body: Some(rusoto_core::ByteStream::new(into_bytes_stream(tokio_file))),
            ..Default::default()
        };
        self.client
            .put_object(put_req)
            .await
            .context("failed to put object")?;
        Ok(file_size)
    }

    /// Get remote S3 file as string
    #[instrument(level = "info")]
    pub async fn get_str(&self, filename: &str) -> anyhow::Result<String> {
        let get_req = rusoto_s3::GetObjectRequest {
            bucket: self.bucket.clone(),
            key: filename.to_string(),
            ..Default::default()
        };
        let file = match self.client.get_object(get_req).await {
            Err(e) => return Err(anyhow::Error::new(e)),
            Ok(x) => x,
        };
        let stream = match file.body {
            Some(x) => x,
            None => return Err(anyhow::Error::msg("stream download error")),
        };
        let body: BytesMut = match stream.map_ok(|b| BytesMut::from(&b[..])).try_concat().await {
            Ok(x) => x,
            Err(e) => return Err(anyhow::Error::new(e)),
        };
        let b: &[u8] = &body[..];
        match std::str::from_utf8(b) {
            Ok(x) => Ok(x.to_string()),
            Err(e) => return Err(anyhow::Error::new(e)),
        }
    }

    /// Save remote S3 file to local file
    #[instrument(level = "info", ret)]
    pub async fn get_file(&self, filename: &str, local_filename: &PathBuf) -> anyhow::Result<u64> {
        let get_req = rusoto_s3::GetObjectRequest {
            bucket: self.bucket.clone(),
            key: filename.to_string(),
            ..Default::default()
        };
        let object = match self.client.get_object(get_req).await {
            Err(e) => return Err(anyhow::Error::new(e)),
            Ok(x) => x,
        };
        let stream = match object.body {
            Some(x) => x,
            None => return Err(anyhow::Error::msg("stream download error")),
        };
        // save stream to local file
        let mut body = stream.into_async_read();
        let mut file = tokio::fs::File::create(local_filename).await?;
        let out = tokio::io::copy(&mut body, &mut file).await?;
        Ok(out)
    }

    #[instrument(ret, level = "info")]
    pub async fn list(&self, prefix: &str) -> anyhow::Result<Vec<S3Object>> {
        let list_req = rusoto_s3::ListObjectsV2Request {
            bucket: self.bucket.clone(),
            prefix: Some(prefix.to_string()),
            ..Default::default()
        };
        let output = self
            .client
            .list_objects_v2(list_req)
            .await
            .context("failed to read object")?;
        // println!("{:#?}", output);
        let mut out = vec![];
        let now = chrono::Utc::now();
        for o in output.contents.unwrap_or_default() {
            out.push(S3Object {
                key: o.key.unwrap_or_default(),
                last_modified: o.last_modified.unwrap_or_default().parse().unwrap_or(now),
                size: o.size.unwrap_or_default(),
            });
        }
        Ok(out)
    }
}
