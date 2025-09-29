// use axum::extract::Multipart;
// use axum::{Json, http::StatusCode};
// use serde::{Deserialize, Serialize};
// use tracing::info;
// use utoipa::ToSchema;

// // TODO: This is good example but remove this later as stream does not accept direct uploads.

// #[derive(Deserialize, ToSchema)]
// #[allow(unused)]
// pub struct UploadForm {
//     pub name: String,
//     #[schema(format = Binary, content_media_type = "application/octet-stream")]
//     pub file: String,
// }

// #[derive(Serialize, ToSchema)]
// pub struct UploadResponse {
//     pub message: String,
//     pub file_name: Option<String>,
//     pub size: usize,
//     pub content_type: Option<String>,
// }

// /// Upload a media file
// #[utoipa::path(
//     post,
//     path = "/upload",
//     request_body(content = UploadForm, content_type = "multipart/form-data"),
//     responses(
//         (status = 200, description = "File uploaded successfully", body = UploadResponse),
//         (status = 400, description = "Bad request", body = super::ErrorResponse),
//         (status = 500, description = "Internal server error", body = super::ErrorResponse)
//     ),
//     tag = "upload"
// )]
// #[tracing::instrument]
// pub async fn upload_file(mut multipart: Multipart) -> Result<Json<UploadResponse>, StatusCode> {
//     info!("Processing file upload request");

//     let mut name: Option<String> = None;
//     let mut content_type: Option<String> = None;
//     let mut size: usize = 0;
//     let mut file_name: Option<String> = None;

//     while let Some(field) = multipart
//         .next_field()
//         .await
//         .map_err(|_| StatusCode::BAD_REQUEST)?
//     {
//         let field_name = field.name();

//         match field_name {
//             Some("name") => {
//                 name = Some(field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?);
//             }
//             Some("file") => {
//                 file_name = field.file_name().map(ToString::to_string);
//                 content_type = field.content_type().map(ToString::to_string);
//                 let bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
//                 size = bytes.len();

//                 // TODO: Save file to storage
//                 // This is where you'd implement actual file storage logic
//             }
//             _ => {
//                 // Skip unknown fields
//                 let _ = field.bytes().await;
//             }
//         }
//     }

//     let response = UploadResponse {
//         message: "File uploaded successfully".to_string(),
//         file_name: file_name.clone(),
//         size,
//         content_type: content_type.clone(),
//     };

//     info!(
//         name = name.as_deref(),
//         content_type = content_type.as_deref(),
//         size = size,
//         file_name = file_name.as_deref(),
//         "File upload completed"
//     );

//     Ok(Json(response))
// }
