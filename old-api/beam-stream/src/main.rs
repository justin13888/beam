pub mod config;
pub mod docs;
pub mod media;
pub mod metrics;
pub mod task;
pub mod utils;

use std::net::SocketAddr;

use axum::{middleware, Router};
use dotenvy::dotenv;
use http::Method;
use listenfd::ListenFd;
use metrics::start_metrics_server;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::{Config, Environment},
    docs::openapi_router,
    media::media_router,
    metrics::track_metrics,
    task::task_router,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load color-eyre
    color_eyre::install()?;

    // Load .env
    dotenv().ok();

    // Load config
    let env = Environment::from_env()?;
    let config = Config::with_env(env.clone())?;
    // TODO: Read config.production_mode, log_level

    // Initialize tracing subscriber
    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "beam_stream=debug,tower_http=debug".into()),
    );
    match &config.production_mode {
        true => subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init(),
        false => subscriber
            .with(tracing_subscriber::fmt::layer().pretty())
            .init(),
    };

    info!("Environment: {:?}", &env);
    info!("Config: {:?}", &config);

    if config.enable_metrics {
        let (_main_server, _metrics_server) = tokio::join!(
            start_main_server(&config.binding_address),
            start_metrics_server(&config.metrics_binding_address)
        );
    } else {
        info!("Metrics server is disabled");
        start_main_server(&config.binding_address).await;
    }

    info!("Shutting down");

    Ok(())
}

async fn start_main_server(address: &SocketAddr) {
    // TODO: implement JWT on all routes
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // TODO: Implement versioning
    let app = Router::new()
        .merge(openapi_router())
        .nest("/task", task_router())
        .nest("/media", media_router())
        .layer(cors)
        .route_layer(middleware::from_fn(track_metrics));

    let mut listenfd = ListenFd::from_env();
    let listener = match listenfd.take_tcp_listener(0).unwrap() {
        // if we are given a tcp listener on listen fd 0, we use that one
        Some(listener) => {
            listener.set_nonblocking(true).unwrap();
            TcpListener::from_std(listener).unwrap()
        }
        // otherwise fall back to local listening
        None => TcpListener::bind(&address).await.unwrap(),
    };

    info!(
        "beam-stream listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
