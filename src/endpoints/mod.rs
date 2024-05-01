pub mod metrics;
pub mod openapi;
pub mod prelude;
pub use prelude::*;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub config_path: String,
}

pub async fn run(listen: &str, config_path: String) -> anyhow::Result<()> {
    use utoipa::Path;

    let shared_state = Arc::new(AppState { config_path });
    let app = Router::new()
        .route(&openapi::__path_handle::path(), get(openapi::handle))
        .route(&metrics::__path_handle::path(), get(metrics::handle))
        .layer(DefaultBodyLimit::disable())
        .layer(Extension(shared_state))
        .layer(axum_trace_full())
        .layer(axum_cors_any())
        .route("/", get(|| async { "# Backups Server API" }));

    axum_serve(listen, app).await
}
