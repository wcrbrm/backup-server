use crate::s3::*;
use serde::Deserialize;
use std::collections::BTreeMap as Map;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(tag = "transport")]
pub enum RealmLocation {
    S3 {
        /// S3 access key
        access_key: String,
        /// S3 secret key
        secret_access_key: String,
        /// S3 bucket name
        bucket: String,
        /// S3 region name
        #[serde(flatten)]
        region: S3Region,
    },
}

impl RealmLocation {
    fn is_s3(&self) -> bool {
        matches!(self, Self::S3 { .. })
    }

    fn get_bucket(&self) -> anyhow::Result<Bucket> {
        match self {
            Self::S3 {
                access_key,
                secret_access_key,
                bucket,
                region,
            } => return Bucket::new(access_key, secret_access_key, bucket, region),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Realm {
    /// what prefix is appended on the upload
    #[serde(default)]
    pub prefix: String,
    /// what name should the uploaded file contain to be identified as a part of the realm
    #[serde(default)]
    pub contains: String,
    #[serde(flatten)]
    pub location: RealmLocation,
    // TODO: lifetime
}

impl Realm {
    pub async fn push(&self, file_path: &PathBuf) -> anyhow::Result<u64> {
        if self.location.is_s3() {
            let bucket = self.location.get_bucket()?;
            if !format!("{}", file_path.display()).contains(&self.contains) {
                anyhow::bail!(
                    "file {} is expected to contain {} to fit the realm",
                    file_path.display(),
                    self.contains
                );
            }

            // remote path is prefix + file name
            let remote_path = format!(
                "{}{}",
                self.prefix,
                file_path.file_name().unwrap().to_str().unwrap()
            );
            return bucket.put_file(&remote_path, file_path).await;
        }

        anyhow::bail!("transport not supported yet")
    }

    pub async fn pull(&self, exchange_dir: &Path) -> anyhow::Result<PathBuf> {
        if self.location.is_s3() {
            let bucket = self.location.get_bucket()?;
            let list = bucket.list(&self.prefix).await?;
            let mut latest = "".to_string();
            for obj in list {
                if !obj.key.contains(&self.contains) {
                    continue;
                }
                latest = obj.key.clone();
            }
            if !latest.is_empty() {
                let local_file_path: PathBuf = Path::new(exchange_dir).join(latest.clone());
                let _ = bucket.get_file(&latest, &local_file_path).await?;
                return Ok(local_file_path);
            }
            anyhow::bail!("no backups")
        }

        anyhow::bail!("transport not supported yet")
    }

    // return stat of the trealm
    pub async fn stat(&self) -> anyhow::Result<(i64, u32, chrono::DateTime<chrono::Utc>)> {
        if self.location.is_s3() {
            let bucket: Bucket = self.location.get_bucket()?;
            let list = bucket.list(&self.prefix).await?;
            let mut total_size = 0;
            let mut total_count = 0u32;
            let mut last_modified = chrono::Utc::now();
            for obj in list {
                if !obj.key.contains(&self.contains) {
                    continue;
                }
                total_size += obj.size;
                total_count += 1;
                last_modified = std::cmp::max(last_modified, obj.last_modified);
            }
            return Ok((total_size, total_count, last_modified));
        }

        anyhow::bail!("transport not supported yet")
    }
}

#[derive(Debug, Deserialize)]
pub struct RealmsConfig {
    pub realms: Map<String, Realm>,
}

// constructor
impl RealmsConfig {
    pub fn from_toml(file_path: &str) -> anyhow::Result<Self> {
        tracing::info!("reading config {}", file_path);
        let out = toml::from_str(&std::fs::read_to_string(&file_path)?)?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let contents = r#"
[realms]

[realms.media]
transport = "S3"
prefix = "project-media"
access_key = ""
secret_access_key = ""
bucket = ""
endpoint = "https://eu2.contabostorage.com"

"#;

        let config: RealmsConfig = toml::from_str(&contents).unwrap();
        println!("{:?}", config);
    }
}
