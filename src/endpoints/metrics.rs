use super::prelude::*;
use super::AppState;
use crate::realms::RealmsConfig;
use lazy_static::lazy_static;
use prometheus::{opts, register_int_gauge, register_int_gauge_vec};
use prometheus::{Encoder, IntGauge, IntGaugeVec, Registry, TextEncoder};

lazy_static! {
    pub static ref UP: IntGauge =
        register_int_gauge!(opts!("up", "Whether the server is running")).unwrap();

    // total number of files
    pub static ref REALM_NUM_FILES: IntGaugeVec = register_int_gauge_vec!(
        opts!("backup_realm_files", "Number of files stored in the realm"),
        &["realm"]
    )
    .expect("Can't create a REALM_NUM_FILES");
    // total size
    pub static ref REALM_SIZE_TOTAL: IntGaugeVec = register_int_gauge_vec!(
        opts!("backup_realm_size_total", "Total estimate of realm size in bytes"),
        &["realm"]
    )
    .expect("Can't create a REALM_SIZE_TOTAL");
    // date of the latest file
    pub static ref REALM_LATEST: IntGaugeVec = register_int_gauge_vec!(
        opts!("backup_realm_timestamp", "Timestamp after the last update"),
        &["realm"]
    )
    .expect("Can't create a REALM_LATEST");
}

#[instrument]
pub async fn to_string(config: &str) -> String {
    let encoder = TextEncoder::new();
    let sr = Registry::new();
    sr.register(Box::new(UP.clone())).unwrap();
    UP.set(1i64);

    sr.register(Box::new(REALM_NUM_FILES.clone())).unwrap();
    sr.register(Box::new(REALM_SIZE_TOTAL.clone())).unwrap();
    sr.register(Box::new(REALM_LATEST.clone())).unwrap();

    match RealmsConfig::from_toml(config) {
        Ok(cfg) => {
            for (key, realm) in cfg.realms {
                match realm.stat().await {
                    Ok(stat) => {
                        REALM_SIZE_TOTAL.with_label_values(&[&key]).set(stat.0);
                        REALM_NUM_FILES
                            .with_label_values(&[&key])
                            .set(stat.1 as i64);
                        REALM_LATEST
                            .with_label_values(&[&key])
                            .set(stat.2.timestamp());
                    }
                    Err(err) => {
                        tracing::warn!("realm {} stat error: {}", key, err.to_string())
                    }
                }
            }
        }
        Err(err) => {
            tracing::warn!("config error: {}", err.to_string())
        }
    }

    let mut buffer = Vec::<u8>::new();
    encoder.encode(&sr.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer.clone()).unwrap()
}

/// Prometheus Metrics
///
/// Prometheus metrics endpoint (health check)
#[utoipa::path(
    get, path = "/stats/backup/metrics", responses(
        (status = 200, description = "Prometheus metrics endpoint (health check)", content_type = "text/plain", body = String),
    ),
)]
pub async fn handle(Extension(shared_state): Extension<Arc<AppState>>) -> impl IntoResponse {
    let config_path = shared_state.clone().config_path.clone();
    to_string(&config_path).await.into_response()
}
