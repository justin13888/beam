// use salvo::prelude::*;
// use serde::{Deserialize, Serialize};
// use salvo::oapi::ToSchema;
// use tracing::info;

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
// #[endpoint(
//     tags("upload"),
//     responses(
//         (status_code = 200, description = "File uploaded successfully"),
//         (status_code = 400, description = "Bad request"),
//         (status_code = 500, description = "Internal server error")
//     )
// )]
// pub async fn upload_file(req: &mut Request, res: &mut Response) {
//     info!("Processing file upload request");
//     // TODO: Implement file upload logic using Salvo's multipart support
// }
