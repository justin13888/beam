pub mod health;
pub mod stream;
pub mod upload;

use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use health::*;
use stream::*;
// use upload::*;

use crate::models::*;

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
        stream_mp4
    ),
    components(schemas(
        HealthStatus,
        ErrorResponse,
        MediaMetadata,
        Title,
        ExternalIdentifiers,
        Ratings,
        ShowMetadata,
        SeasonMetadata,
        ShowDates,
        EpisodeMetadata,
        MovieMetadata,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        // (name = "upload", description = "File upload operations"), 
        (name = "stream", description = "Streaming endpoints")
    )
)]
pub struct ApiDoc;

/// Create the main router with all routes
pub fn create_router() -> OpenApiRouter {
    OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health_check))
        // .routes(routes!(upload_file))
        .routes(routes!(stream_mp4))
}
