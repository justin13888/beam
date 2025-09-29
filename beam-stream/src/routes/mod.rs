pub mod health;
pub mod media;
pub mod upload;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use health::*;
use media::*;
// use upload::*;

/// Main API documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Beam Stream API",
        version = "1.0.0",
        description = "A high-performance media streaming API"
    ),
    paths(
        health_check,
        // upload_file,
        get_media_info,
        stream_media
    ),
    components(schemas(
        HealthStatus,
        // UploadResponse,
        MediaInfo,
        ErrorResponse
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        // (name = "upload", description = "File upload operations"), 
        (name = "media", description = "Media streaming and information")
    )
)]
pub struct ApiDoc;

/// Create the main router with all routes
pub fn create_router() -> OpenApiRouter {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health_check))
        // .routes(routes!(upload_file))
        .routes(routes!(get_media_info))
        .routes(routes!(stream_media))
}
