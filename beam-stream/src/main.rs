mod models;
mod routes;

use std::sync::atomic::Ordering;

use eyre::{Result, eyre};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_scalar::{Scalar, Servable};

use beam_stream::config::Config;
use routes::create_router;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
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

    info!("Starting beam-stream...");

    // Load configuration
    let config = Config::from_env().map_err(|e| eyre!(e))?;

    info!("Configuration loaded: {:?}", config);

    // Ensure video and cache directories exist
    tokio::fs::create_dir_all(config.video_dir)
        .await
        .expect("Failed to create video directory");
    tokio::fs::create_dir_all(config.cache_dir)
        .await
        .expect("Failed to create cache directory");

    // Initialize ffmpeg bindings
    ffmpeg_next::init().map_err(|e| eyre!("Failed to initialize ffmpeg: {}", e))?;

    // Initialize m3u8-rs static variables
    m3u8_rs::WRITE_OPT_FLOAT_PRECISION.store(5, Ordering::Relaxed);

    let (router, api) = create_router().split_for_parts();
    let router = router.merge(Scalar::with_url("/openapi", api));

    let app = router.into_make_service();

    info!("Binding to address: {}", config.bind_address);
    let listener = tokio::net::TcpListener::bind(&config.bind_address)
        .await
        .unwrap();
    let local_addr = listener.local_addr().unwrap();

    info!("Server listening on http://{}", local_addr);
    info!(
        "API documentation available at http://{}/openapi",
        local_addr
    );

    axum::serve(listener, app)
        .await
        .map_err(|e| eyre!("Server error: {}", e))?;

    Ok(())
}
