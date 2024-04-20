use std::sync::Arc;

use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;

use super::{MediaItem, MediaLibrary};

pub fn search_router(store: Arc<Mutex<MediaLibrary>>) -> Router {
    Router::new().merge(
        Router::new()
            // .route("/", get(search_media))
            .with_state(store),
    )
}

// TODO: Implement search endpoint
