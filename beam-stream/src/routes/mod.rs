pub mod auth;
pub mod health;
pub mod stream;
pub mod upload;

use salvo::prelude::*;

pub use health::*;
pub use stream::*;

/// Create the main API router with all routes
pub fn create_router() -> Router {
    Router::new()
        .push(Router::with_path("health").get(health_check))
        .push(Router::with_path("stream/<id>/token").post(get_stream_token))
        .push(Router::with_path("stream/mp4/<id>").get(stream_mp4))
}
