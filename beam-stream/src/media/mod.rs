pub mod collection;
pub mod search;

use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use hyper::StatusCode;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use tokio::sync::Mutex;
use url::Url;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct MediaLibrary(Vec<MediaItem>);

// TODO: implement endpoints

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
#[serde(tag = "type")]
pub enum MediaItem {
    Movie(MovieItem),
    Series(SeriesItem),
}

impl MediaItem {
    pub fn id(&self) -> &str {
        match self {
            MediaItem::Movie(item) => &item.id,
            MediaItem::Series(item) => &item.id,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct MovieItem {
    pub id: String,
    pub title: String,
    pub metadata: Option<MovieMetadata>,
    /// List of paths to all versions of the movie
    pub movie: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct MovieMetadata {
    // pub duration: u32,
    // pub release_date: DateTime<Utc>,
    // pub genres: Vec<String>,
    // pub rating: Option<u8>,
    // pub thumbnail_url: Option<Url>,
    // pub audio_languages: Vec<String>,
    // pub subtitles: Vec<String>, // TODO: Represent as struct { language: String, name: String }
} // TODO

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct SeriesItem {
    pub id: String,
    pub title: String,
    pub metadata: Option<SeriesMetadata>,
    /// List of episodes, each containing a list of paths to all
    /// versions of the episode
    pub episodes: Vec<EpisodeItem>,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct EpisodeItem {
    pub episode_number: u32,
    pub season_number: u32,
    pub metadata: Option<SeriesMetadata>,
    /// List of episodes, each containing a list of paths to all
    /// versions of the episode
    pub versions: Vec<PathBuf>,
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct SeriesMetadata; // TODO

type Store = Mutex<MediaLibrary>;

pub fn media_router() -> Router {
    // TODO: This is for testing
    let store = Arc::new(Mutex::new(MediaLibrary(vec![
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
    ])));

    Router::new()
        .merge(
            Router::new()
                .route("/:id", get(get_media))
                .route("/:id/related", get(get_related_media))
                .route("/:id/history", get(get_media_history))
                .with_state(store.clone()),
        )
        .nest("/collection", collection::collection_router())
        .nest("/search", search::search_router(store.clone()))
}

/// Get MediaItem by id
#[utoipa::path(
    get,
    path = "/media/{id}",
    responses(
        (status = 200, description = "MediaItem found", body = MediaItem),
        (status = 404, description = "MediaItem not found", body = TaskError, example = json!(MediaItemError::NotFound(String::from("id = abc"))))
    ),
    params(
        ("id" = string, Path, description = "Media Item ID", example = "abc")
    )
)]
async fn get_media(Path(id): Path<String>, State(store): State<Arc<Store>>) -> impl IntoResponse {
    let tasks = store.lock().await;

    // Find MediaItem by id
    let result = tasks.0.iter().find(|&task| task.id() == &id);

    // Return response
    match result {
        Some(task) => (StatusCode::OK, Json(task)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(MediaItemError::NotFound(format!("id = {id}"))),
        )
            .into_response(),
    }
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct WatchHistory {
    pub id: String,
    pub user_id: String,
    // #[serde(serialize_with = "serialize_episode_map")]
    pub progress: Option<HashMap<Episode, VideoProgress>>,
    /// List of paths to all versions of the movie
    pub movie: Vec<PathBuf>,
}

// // Custom serializer for HashMap<Episode, VideoProgress>
// fn serialize_episode_map<S>(
//     episode_map: &HashMap<Episode, VideoProgress>,
//     serializer: S,
// ) -> Result<S::Ok, S::Error>
// where
//     S: Serializer,
// {
//     let mut map = serializer.serialize_map(Some(episode_map.len()))?;
//     for (k, v) in episode_map {
//         let key_as_string = format!("S{}E{}", k.season_number, k.episode_number);
//         map.serialize_entry(&key_as_string, v)?;
//     }
//     map.end()
// }

#[derive(ToSchema, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Episode {
    pub episode_number: u32,
    pub season_number: u32,
}

impl Serialize for Episode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let episode_str = format!("S{}E{}", self.season_number, self.episode_number);
        serializer.serialize_str(&episode_str)
    }
}

impl<'de> Deserialize<'de> for Episode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EpisodeVisitor;

        impl<'de> Visitor<'de> for EpisodeVisitor {
            type Value = Episode;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string in the format S{season_number}E{episode_number}")
            }

            fn visit_str<E>(self, value: &str) -> Result<Episode, E>
            where
                E: de::Error,
            {
                let parts: Vec<&str> = value.split('E').collect();
                if parts.len() != 2 {
                    return Err(E::invalid_value(Unexpected::Str(value), &self));
                }
                let season_number = parts[0]
                    .trim_start_matches('S')
                    .parse::<u32>()
                    .map_err(|_| E::invalid_value(Unexpected::Str(value), &self))?;
                let episode_number = parts[1]
                    .parse::<u32>()
                    .map_err(|_| E::invalid_value(Unexpected::Str(value), &self))?;

                Ok(Episode {
                    season_number,
                    episode_number,
                })
            }
        }

        deserializer.deserialize_str(EpisodeVisitor)
    }
}

#[derive(Serialize, Deserialize, ToSchema, PartialEq, Eq, Debug, Clone)]
pub struct VideoProgress {
    pub progress: u32,
    pub last_watched: DateTime<Utc>,
}

/// Get related MediaItems for some MediaItem ID
#[utoipa::path(
    get,
    path = "/media/{id}/related",
    responses(
        (status = 200, description = "MediaItem found", body = Vec<MediaItem>),
        (status = 404, description = "MediaItem not found", body = TaskError, example = json!(MediaItemError::NotFound(String::from("id = abc"))))
    ),
    params(
        ("id" = string, Path, description = "Media Item ID", example = "abc")
    )
)]
async fn get_related_media(
    Path(id): Path<String>,
    State(store): State<Arc<Store>>,
) -> impl IntoResponse {
    let tasks = store.lock().await;

    // Find MediaItem by id
    let result = tasks.0.iter().find(|&task| task.id() == &id);

    // Return related media if found
    // TODO: Implement. Currently just returns everything
    match result {
        Some(_task) => (StatusCode::OK, Json(tasks.clone())).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(MediaItemError::NotFound(format!("id = {id}"))),
        )
            .into_response(),
    }
}

/// Get watch history of MediaItem by ID
#[utoipa::path(
    get,
    path = "/media/{id}/history",
    responses(
        (status = 200, description = "MediaItem found", body = WatchHistory),
        (status = 404, description = "MediaItem not found", body = TaskError, example = json!(MediaItemError::NotFound(String::from("id = abc"))))
    ),
    params(
        ("id" = string, Path, description = "Media Item ID", example = "abc")
    )
)]
async fn get_media_history(
    Path(_id): Path<String>,
    State(_store): State<Arc<Store>>,
) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(WatchHistory {
            id: "abc".to_string(),
            user_id: "def".to_string(),
            progress: Some(
                [(
                    Episode {
                        episode_number: 1,
                        season_number: 1,
                    },
                    VideoProgress {
                        progress: 50,
                        last_watched: Utc::now(),
                    },
                )]
                .iter()
                .cloned()
                .collect(),
            ),
            movie: [PathBuf::from("test.mp4")].to_vec(),
        }),
    )
        .into_response()

    // let tasks = store.lock().await;

    // // Find MediaItem by id
    // let result = tasks.0.iter().find(|&task| task.id() == &id);

    // // Return related media if found
    // // TODO: Implement. Currently just returns everything
    // match result {
    //     Some(_task) => (StatusCode::OK, Json(tasks.clone())).into_response(),
    //     None => (
    //         StatusCode::NOT_FOUND,
    //         Json(MediaItemError::NotFound(format!("id = {id}"))),
    //     )
    //         .into_response(),
    // }
}

/// MediaItem operation errors
#[derive(Serialize, Deserialize, ToSchema)]
pub enum MediaItemError {
    /// Movie not found
    #[schema(example = "id = abc")]
    NotFound(String),
}
