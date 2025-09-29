use axum::extract::{Path, Query};
use axum::response::Response;
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use tracing::info;
use utoipa::{IntoParams, ToSchema};

use beam_stream::utils::metadata::extract_metadata;

// TODO: Expand this to all the metadata actually needed by the server
#[derive(Serialize, ToSchema)]
pub struct MediaInfo {
    pub file_name: String,
    pub duration: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bitrate: Option<u32>,
    pub format: Option<String>,
    pub size: u64,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct StreamParams {
    pub start: Option<f64>,
    pub duration: Option<f64>,
    pub quality: Option<String>,
}

/// Get media metadata by ID
#[utoipa::path(
    get,
    path = "/media/{id}/metadata",
    params(
        ("id" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Media information retrieved", body = MediaInfo),
        (status = 404, description = "Media not found", body = super::ErrorResponse),
        (status = 500, description = "Internal server error", body = super::ErrorResponse)
    ),
    tag = "media"
)]
#[tracing::instrument]
pub async fn get_media_metadata(Path(id): Path<String>) -> Result<Json<MediaInfo>, StatusCode> {
    info!("Getting media metadata for ID: {}", id);

    // Construct the file path - in a real implementation, you'd map IDs to actual file paths
    let file_path = PathBuf::from("videos").join(&id);

    if !file_path.exists() {
        return Err(StatusCode::NOT_FOUND);
    }

    match extract_metadata(&file_path) {
        Ok(metadata) => {
            let mut width = None;
            let mut height = None;
            let mut bitrate = None;
            let mut format = None;

            // Get video information from the best video stream
            if let Some(video_stream_idx) = metadata.best_video_stream
                && let Some(stream) = metadata.streams.get(video_stream_idx)
                && let Some(video) = &stream.video
            {
                width = Some(video.width);
                height = Some(video.height);
                bitrate = Some(video.bit_rate as u32);
                format = Some(video.codec_name.clone());
            }

            let media_metadata = MediaInfo {
                file_name: id,
                duration: Some(metadata.duration_seconds()),
                width,
                height,
                bitrate,
                format,
                size: metadata.file_size,
            };

            Ok(Json(media_metadata))
        }
        Err(err) => {
            tracing::error!("Failed to extract metadata: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Stream media file
#[utoipa::path(
    get,
    path = "/media/{id}/stream",
    params(
        ("id" = String, Path, description = "Media ID"),
        StreamParams
    ),
    responses(
        (status = 200, description = "Media stream", content_type = "video/mp4"),
        (status = 404, description = "File not found", body = super::ErrorResponse),
        (status = 416, description = "Range not satisfiable", body = super::ErrorResponse),
        (status = 500, description = "Internal server error", body = super::ErrorResponse)
    ),
    tag = "media"
)]
#[tracing::instrument]
pub async fn stream_media(
    Path(id): Path<String>,
    Query(params): Query<StreamParams>,
) -> Result<Response, StatusCode> {
    info!(
        "Streaming media with ID: {}, params: start={:?}, duration={:?}, quality={:?}",
        id, params.start, params.duration, params.quality
    );

    // TODO: Implement actual streaming logic
    // This is where you'd:
    // 1. Check if file exists
    // 2. Handle range requests for partial content
    // 3. Transcode if needed based on quality parameter
    // 4. Stream the content

    Err(StatusCode::NOT_IMPLEMENTED)
}
