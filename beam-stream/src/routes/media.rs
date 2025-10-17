use axum::extract::Path;
use axum::{Json, http::StatusCode};
use tracing::info;

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
    info!("Getting media metadata for ID: {}", id);

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

// #[derive(Debug, Deserialize, IntoParams)]
// pub struct StreamParams {
//     pub start: Option<f64>,
//     pub duration: Option<f64>,
//     pub quality: Option<String>,
// }

// /// Stream media by ID
// #[utoipa::path(
//     get,
//     path = "/media/{id}/stream",
//     params(
//         ("id" = String, Path, description = "Media ID"),
//         StreamParams
//     ),
//     responses(
//         (status = 200, description = "Media stream", content_type = "video/mp4"),
//         (status = 404, description = "File not found", body = super::ErrorResponse),
//         (status = 416, description = "Range not satisfiable", body = super::ErrorResponse),
//         (status = 500, description = "Internal server error", body = super::ErrorResponse)
//     ),
//     tag = "media"
// )]
// #[tracing::instrument]
// pub async fn stream_media(
//     Path(id): Path<String>,
//     Query(params): Query<StreamParams>,
//     headers: HeaderMap,
// ) -> Result<Response, StatusCode> {
//     info!(
//         "Streaming media with ID: {}, params: start={:?}, duration={:?}, quality={:?}",
//         id, params.start, params.duration, params.quality
//     );

//     // Construct the file path - same as metadata endpoint
//     let file_path = PathBuf::from("videos").join(&id);

//     if !file_path.exists() {
//         return Err(StatusCode::NOT_FOUND);
//     }

//     // Get file metadata
//     let file_metadata = match std::fs::metadata(&file_path) {
//         Ok(metadata) => metadata,
//         Err(err) => {
//             tracing::error!("Failed to get file metadata: {:?}", err);
//             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//         }
//     };

//     let file_size = file_metadata.len();

//     // Determine content type from file extension
//     let content_type = match file_path.extension().and_then(|ext| ext.to_str()) {
//         Some("mp4") => "video/mp4",
//         Some("avi") => "video/x-msvideo",
//         Some("mov") => "video/quicktime",
//         Some("mkv") => "video/x-matroska",
//         Some("webm") => "video/webm",
//         Some("flv") => "video/x-flv",
//         Some("wmv") => "video/x-ms-wmv",
//         Some("m4v") => "video/mp4",
//         _ => "application/octet-stream",
//     };

//     // Handle range requests
//     let range = headers.get("range");
//     let (start, end, status_code) = if let Some(range_header) = range {
//         let range_str = match range_header.to_str() {
//             Ok(s) => s,
//             Err(_) => return Err(StatusCode::BAD_REQUEST),
//         };

//         if !range_str.starts_with("bytes=") {
//             return Err(StatusCode::BAD_REQUEST);
//         }

//         let range_part = &range_str[6..]; // Remove "bytes="
//         let parts: Vec<&str> = range_part.split('-').collect();

//         if parts.len() != 2 {
//             return Err(StatusCode::BAD_REQUEST);
//         }

//         let start = if parts[0].is_empty() {
//             // Suffix range like "-500"
//             if let Ok(suffix) = parts[1].parse::<u64>() {
//                 if suffix >= file_size {
//                     0
//                 } else {
//                     file_size - suffix
//                 }
//             } else {
//                 return Err(StatusCode::BAD_REQUEST);
//             }
//         } else if let Ok(s) = parts[0].parse::<u64>() {
//             s
//         } else {
//             return Err(StatusCode::BAD_REQUEST);
//         };

//         let end = if parts[1].is_empty() {
//             file_size - 1
//         } else if let Ok(e) = parts[1].parse::<u64>() {
//             std::cmp::min(e, file_size - 1)
//         } else {
//             return Err(StatusCode::BAD_REQUEST);
//         };

//         if start > end || start >= file_size {
//             return Err(StatusCode::RANGE_NOT_SATISFIABLE);
//         }

//         (start, end, StatusCode::PARTIAL_CONTENT)
//     } else {
//         (0, file_size - 1, StatusCode::OK)
//     };

//     // Open file and seek to start position
//     let mut file = match File::open(&file_path).await {
//         Ok(f) => f,
//         Err(err) => {
//             tracing::error!("Failed to open file: {:?}", err);
//             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//         }
//     };

//     // Seek to start position if needed
//     if start > 0 {
//         use tokio::io::AsyncSeekExt;
//         if let Err(err) = file.seek(std::io::SeekFrom::Start(start)).await {
//             tracing::error!("Failed to seek in file: {:?}", err);
//             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//         }
//     }

//     let content_length = end - start + 1;

//     // Read the requested range
//     let mut buffer = vec![0u8; content_length as usize];
//     match file.read_exact(&mut buffer).await {
//         Ok(_) => {}
//         Err(err) => {
//             tracing::error!("Failed to read file: {:?}", err);
//             return Err(StatusCode::INTERNAL_SERVER_ERROR);
//         }
//     }

//     // Build response
//     let mut response = Response::builder()
//         .status(status_code)
//         .header("Content-Type", content_type)
//         .header("Content-Length", content_length.to_string())
//         .header("Accept-Ranges", "bytes");

//     // Add range headers for partial content
//     if status_code == StatusCode::PARTIAL_CONTENT {
//         response = response.header(
//             "Content-Range",
//             format!("bytes {}-{}/{}", start, end, file_size),
//         );
//     }

//     // Add cache headers
//     response = response
//         .header("Cache-Control", "public, max-age=3600")
//         .header("ETag", format!("\"{}\"", file_size)); // Simple ETag based on file size

//     match response.body(Body::from(buffer)) {
//         Ok(resp) => Ok(resp),
//         Err(err) => {
//             tracing::error!("Failed to build response: {:?}", err);
//             Err(StatusCode::INTERNAL_SERVER_ERROR)
//         }
//     }
// }
