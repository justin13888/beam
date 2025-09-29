use axum::extract::Multipart;
use serde::Deserialize;
use std::env;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_scalar::{Scalar, Servable};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Initialize JSON logging
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .json()
                .with_current_span(false)
                .with_span_list(true),
        )
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("beam_stream=info,tower_http=debug,axum=debug")),
        )
        .init();

    info!("Starting beam-stream service");

    let (router, api) = OpenApiRouter::new()
        .routes(routes!(hello_form))
        .split_for_parts();

    let router = router.merge(Scalar::with_url("/openapi", api));

    let app = router.into_make_service();

    // Get bind address from environment variable
    let bind_addr = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    info!("Binding to address: {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();

    info!("Server listening on http://{}", local_addr);
    info!(
        "API documentation available at http://{}/openapi",
        local_addr
    );

    axum::serve(listener, app).await
}

/// Just a schema for axum native multipart
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct HelloForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

#[utoipa::path(
    post,
    path = "/hello",
    request_body(content = HelloForm, content_type = "multipart/form-data")
)]
#[tracing::instrument]
async fn hello_form(mut multipart: Multipart) -> String {
    info!("Processing multipart form request");

    let mut name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut size: usize = 0;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name();

        match &field_name {
            Some("name") => {
                name = Some(field.text().await.expect("should be text for name field"));
            }
            Some("file") => {
                file_name = field.file_name().map(ToString::to_string);
                content_type = field.content_type().map(ToString::to_string);
                let bytes = field.bytes().await.expect("should be bytes for file field");
                size = bytes.len();
            }
            _ => (),
        };
    }

    let response = format!(
        "name: {}, content_type: {}, size: {}, file_name: {}",
        name.as_deref().unwrap_or_default(),
        content_type.as_deref().unwrap_or_default(),
        size,
        file_name.as_deref().unwrap_or_default()
    );

    info!(
        name = name.as_deref(),
        content_type = content_type.as_deref(),
        size = size,
        file_name = file_name.as_deref(),
        "Processed multipart form"
    );

    response
}
