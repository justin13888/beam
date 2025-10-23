use std::path::PathBuf;

use axum::body::Body;
use axum::extract::Path;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::{Json, http::StatusCode};
use beam_stream::utils::cache::generate_mp4_cache;
use tokio::fs::File;
use tracing::{debug, error, trace};

use crate::models::{MediaMetadata, SeasonMetadata, ShowDates, ShowMetadata, Title};

/// Get media metadata by ID
#[utoipa::path(
    get,
    path = "/media/{id}/metadata",
    params(
        ("id" = String, Path, description = "Media ID")
    ),
    responses(
        (status = 200, description = "Media information retrieved", body = MediaMetadata),
        (status = 404, description = "Media not found", body = super::ErrorResponse),
        (status = 500, description = "Internal server error", body = super::ErrorResponse)
    ),
    tag = "media"
)]
#[tracing::instrument]
pub async fn get_media_metadata(Path(id): Path<String>) -> Result<Json<MediaMetadata>, StatusCode> {
    debug!("Getting media metadata for ID: {}", id);

    let media_metadata = MediaMetadata::Show(ShowMetadata {
        title: Title {
            original: String::from("Unknown Title"),
            localized: None,
            alternatives: None,
        },
        description: None,
        year: None,
        seasons: vec![SeasonMetadata {
            season_number: 1,
            dates: ShowDates {
                first_aired: None,
                last_aired: None,
            },
            episode_runtime: None,
            episodes: vec![],
            poster_url: None,
            genres: vec![],
            ratings: None,
        }],
        identifiers: None,
    });

    Ok(Json(media_metadata))
}

/// Stream media by ID - serves AVFoundation-friendly fragmented MP4
#[utoipa::path(
    get,
    path = "/media/{id}/stream",
    params(
        ("id" = String, Path, description = "Media ID")
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
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    debug!("Streaming media with ID: {}", id);

    // TODO: Map ID to actual video file path
    // For now, hardcode to test.mkv
    let source_video_path = PathBuf::from("videos/test.mkv");
    let cache_mp4_path = PathBuf::from("cache/test.mp4");

    if !source_video_path.exists() {
        error!("Source video file not found: {:?}", source_video_path);
        return Err(StatusCode::NOT_FOUND);
    }

    // // Ensure cache directory exists
    // // don't need this because cache dir is created at startup
    // if let Some(parent) = cache_mp4_path.parent() && let Err(err) = tokio::fs::create_dir_all(parent).await {
    //         error!("Failed to create cache directory: {:?}", err);
    //         return Err(StatusCode::INTERNAL_SERVER_ERROR);
    // }

    // Generate MP4 if it doesn't exist or is outdated
    if !cache_mp4_path.exists() {
        trace!("Cached MP4 not found, generating: {:?}", cache_mp4_path);

        if let Err(err) = generate_mp4_cache(&source_video_path, &cache_mp4_path).await {
            error!("Failed to generate MP4: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }

        trace!("MP4 generation complete: {:?}", cache_mp4_path);
    } else {
        trace!("Using cached MP4: {:?}", cache_mp4_path);
    }

    // Serve the MP4 file with range request support
    serve_mp4_file(&cache_mp4_path, &headers).await
}

/// Serve MP4 file with HTTP range request support for AVFoundation
async fn serve_mp4_file(file_path: &PathBuf, headers: &HeaderMap) -> Result<Response, StatusCode> {
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    // Get file metadata
    let file_metadata = match tokio::fs::metadata(file_path).await {
        Ok(metadata) => metadata,
        Err(err) => {
            error!("Failed to get file metadata: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let file_size = file_metadata.len();

    // Always use video/mp4 content type since we're serving MP4
    let content_type = "video/mp4";

    // Handle range requests
    let range = headers.get("range");
    let (start, end, status_code) = if let Some(range_header) = range {
        let range_str = match range_header.to_str() {
            Ok(s) => s,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        };

        if !range_str.starts_with("bytes=") {
            return Err(StatusCode::BAD_REQUEST);
        }

        let range_part = &range_str[6..]; // Remove "bytes="
        let parts: Vec<&str> = range_part.split('-').collect();

        if parts.len() != 2 {
            return Err(StatusCode::BAD_REQUEST);
        }

        let start = if parts[0].is_empty() {
            // Suffix range like "-500"
            if let Ok(suffix) = parts[1].parse::<u64>() {
                if suffix >= file_size {
                    0
                } else {
                    file_size - suffix
                }
            } else {
                return Err(StatusCode::BAD_REQUEST);
            }
        } else if let Ok(s) = parts[0].parse::<u64>() {
            s
        } else {
            return Err(StatusCode::BAD_REQUEST);
        };

        let end = if parts[1].is_empty() {
            file_size - 1
        } else if let Ok(e) = parts[1].parse::<u64>() {
            std::cmp::min(e, file_size - 1)
        } else {
            return Err(StatusCode::BAD_REQUEST);
        };

        if start > end || start >= file_size {
            return Err(StatusCode::RANGE_NOT_SATISFIABLE);
        }

        (start, end, StatusCode::PARTIAL_CONTENT)
    } else {
        (0, file_size - 1, StatusCode::OK)
    };

    // Open file and seek to start position
    let mut file = match File::open(file_path).await {
        Ok(f) => f,
        Err(err) => {
            error!("Failed to open file: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Seek to start position if needed
    if start > 0 {
        if let Err(err) = file.seek(std::io::SeekFrom::Start(start)).await {
            error!("Failed to seek in file: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    let content_length = end - start + 1;

    // Read the requested range
    let mut buffer = vec![0u8; content_length as usize];
    match file.read_exact(&mut buffer).await {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to read file: {:?}", err);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Build response
    let mut response = Response::builder()
        .status(status_code)
        .header("Content-Type", content_type)
        .header("Content-Length", content_length.to_string())
        .header("Accept-Ranges", "bytes");

    // Add range headers for partial content
    if status_code == StatusCode::PARTIAL_CONTENT {
        response = response.header(
            "Content-Range",
            format!("bytes {}-{}/{}", start, end, file_size),
        );
    }

    // Add cache headers for better performance
    response = response
        .header("Cache-Control", "public, max-age=3600")
        .header("ETag", format!("\"{}\"", file_size)); // Simple ETag based on file size

    match response.body(Body::from(buffer)) {
        Ok(resp) => Ok(resp),
        Err(err) => {
            error!("Failed to build response: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
