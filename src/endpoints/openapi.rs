use super::metrics;
use super::prelude::*;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(metrics::handle,), components(schemas(HttpErrMessage,)))]
pub struct ApiDoc;

/// returns OpenAPI documentation builder, to be used as string or server JSON response
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Open API
///
/// openapi.json endpoint
#[utoipa::path(
    get, path = "/stat/backup/openapi.json", 
    responses(
        (status = 200, description = "returns open api the service", content_type = "application/json"),
    ),
)]
pub async fn handle() -> impl IntoResponse {
    Json(openapi())
}
