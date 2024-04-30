pub mod metrics;
pub mod openapi;
pub mod prelude;
pub use prelude::*;

#[derive(Debug, Clone)]
struct AppState {
    config_path: String,
}

pub async fn run(listen: &str, config_path: String) -> anyhow::Result<()> {
    let shared_state = Arc::new(Mutex::new(AppState { config_path }));
    let app = Router::new()
        .route("/openapi.json", get(openapi::handle))
        .route("/stat/backup/metrics", get(metrics::handle))
        .layer(DefaultBodyLimit::disable())
        .layer(Extension(shared_state))
        .layer(axum_trace_full())
        .layer(axum_cors_any())
        .route("/", get(|| async { "# Backups Server API" }));

    axum_serve(listen, app).await
}
