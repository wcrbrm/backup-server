use super::prelude::*;

use lazy_static::lazy_static;
use prometheus::{opts, register_int_gauge};
#[allow(unused_imports)]
use prometheus::{Encoder, Gauge, IntGauge, Opts, Registry, TextEncoder};

lazy_static! {
    pub static ref UP: IntGauge =
        register_int_gauge!(opts!("up", "Whether the server is running")).unwrap();
}

#[instrument]
pub fn to_string() -> String {
    let encoder = TextEncoder::new();
    // let labels = HashMap::new();
    // let sr = Registry::new_custom(Some("api".to_string()), Some(labels)).unwrap();
    let sr = Registry::new();
    sr.register(Box::new(UP.clone())).unwrap();
    UP.set(1i64);

    let mut buffer = Vec::<u8>::new();
    encoder.encode(&sr.gather(), &mut buffer).unwrap();
    String::from_utf8(buffer.clone()).unwrap()
}


/// Prometheus Metrics
///
/// Prometheus metrics endpoint (health check)
#[utoipa::path(
    get, path = "/v3/api/merchants/metrics", responses(
        (status = 200, description = "Prometheus metrics endpoint (health check)", content_type = "text/plain", body = String),
    ),
)]
pub async fn handle() -> impl IntoResponse {
    to_string().into_response()
}
