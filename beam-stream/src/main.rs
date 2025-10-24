use std::sync::atomic::Ordering;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{
    Router,
    response::{Html, IntoResponse},
    routing::get,
};
use eyre::{Result, eyre};
use listenfd::ListenFd;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use utoipa_scalar::{Scalar, Servable};

use crate::config::Config;
use routes::create_router;

use crate::graphql::create_schema;

pub mod config;
mod graphql;
mod routes;

const GRAPHQL_PATH: &str = "/graphql";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Initialize JSON logging
    beam_stream::logging::init_tracing();

    info!("Starting beam-stream...");

    // Load configuration
    let config = Config::from_env().map_err(|e| eyre!(e))?;

    info!("Configuration loaded: {:?}", config);

    // Ensure video and cache directories exist
    tokio::fs::create_dir_all(&config.video_dir)
        .await
        .expect("Failed to create video directory");
    tokio::fs::create_dir_all(&config.cache_dir)
        .await
        .expect("Failed to create cache directory");

    // Initialize ffmpeg bindings
    ffmpeg_next::init().map_err(|e| eyre!("Failed to initialize ffmpeg: {}", e))?;

    // Initialize m3u8-rs static variables
    m3u8_rs::WRITE_OPT_FLOAT_PRECISION.store(5, Ordering::Relaxed);

    let (router, api) = create_router().split_for_parts();
    let router = router.merge(Scalar::with_url("/openapi", api));

    let schema = create_schema(&config);
    let graphql_router =
        Router::new().route("/graphql", get(graphiql).post_service(GraphQL::new(schema)));

    let app = router
        .merge(graphql_router)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any), // TODO: Restrict headers as necessary
        )
        .into_make_service();

    info!("Binding to address: {}", config.bind_address);
    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        // if we are given a tcp listener on listen fd 0, we use that one
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => TcpListener::bind(&config.bind_address).await.unwrap(),
    };
    let local_addr = listener.local_addr().unwrap();

    info!("Server listening on http://{local_addr}");
    info!("API documentation available at http://{local_addr}/openapi",);
    info!("GraphiQL interface available at http://{local_addr}/{GRAPHQL_PATH}",);

    axum::serve(listener, app)
        .await
        .map_err(|e| eyre!("Server error: {}", e))?;

    Ok(())
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint(GRAPHQL_PATH).finish())
}
