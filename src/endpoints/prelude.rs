pub use axum::{extract::DefaultBodyLimit, extract::Extension, routing::*, Router};
// pub use axum::debug_handler;
pub use axum::response::*;
pub use std::sync::{Arc, Mutex};
pub use tower_http::classify::*;
pub use tower_http::cors::{Any, CorsLayer};
// pub use tower_http::limit::*;
pub use tower_http::trace::TraceLayer;
pub use tower_http::trace::*;
pub use tracing::*;
pub use axum::Json;
pub use utoipa::ToSchema;
pub use serde::Serialize;

#[derive(Serialize, ToSchema)]
pub struct HttpErrMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    message: String,
}

pub fn axum_cors_any() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}

pub fn axum_trace_full() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .level(Level::INFO)
                .include_headers(true),
        )
        .on_request(DefaultOnRequest::new().level(Level::TRACE))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .include_headers(true),
        )
}


pub async fn axum_serve(listen: &str, app: axum::Router) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(listen).await.unwrap();
    println!("Listening on {}", listen);
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
