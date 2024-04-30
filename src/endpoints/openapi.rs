use super::prelude::*;
use super::metrics;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        metrics::handle,
    ),
    components(schemas(HttpErrMessage,))
)]
pub struct ApiDoc;

/// returns OpenAPI documentation builder, to be used as string or server JSON response
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// */openapi.json endpoint
pub async fn handle() -> impl IntoResponse {
    Json(openapi())
}
