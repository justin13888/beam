use std::{path::PathBuf, sync::Arc};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use utoipa::ToSchema;

use super::{EpisodeItem, MediaItem, MediaItemFilter, MediaItemSort, MovieItem, SeriesItem};

type Store = Mutex<Vec<MediaCollection>>;

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct MediaCollection {
    pub name: String,
    pub description: String,
    pub media: Vec<MediaItem>,
}

pub fn collection_router() -> Router {
    let store = Arc::new(Mutex::new(vec![MediaCollection {
        name: "Test Collection".to_string(),
        description: "A test collection".to_string(),
        media: vec![
            MediaItem::Movie(MovieItem {
                id: "abc".to_string(),
                title: "Test movie".to_string(),
                metadata: None,
                movie: [PathBuf::from("test.mp4")].to_vec(),
            }),
            MediaItem::Series(SeriesItem {
                id: "cde".to_string(),
                title: "Test series".to_string(),
                metadata: None,
                episodes: [EpisodeItem {
                    episode_number: 1,
                    season_number: 1,
                    metadata: None,
                    versions: [PathBuf::from("test.mp4")].to_vec(),
                }]
                .to_vec(),
            }),
        ],
    }]));

    Router::new().merge(
        Router::new()
            .route("/:id", post(query_collection))
            .with_state(store),
    )
}

/// Collection search query
#[derive(Deserialize, Serialize, ToSchema)]
pub struct CollectionSearchQuery {
    /// Search by date range
    filter: Option<MediaItemFilter>,

    // Sort by
    sort: Option<MediaItemSort>,
    // TODO: Implement cursor based pagination
}

/// Search MediaCollection by sort and filter
#[utoipa::path(
    post,
    path = "/media/collection/{id}",
    request_body = CollectionSearchQuery,
    responses(
        (status = 200, description = "List matching MediaItems by query", body = [MediaItem])
    )
)]
async fn query_collection(
    State(_store): State<Arc<Store>>,
    Json(_query): Json<CollectionSearchQuery>,
) -> Json<Vec<MediaItem>> {
    // TODO: Implement

    Json(vec![])
}
