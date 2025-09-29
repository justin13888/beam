mod routes;

use std::env;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_scalar::{Scalar, Servable};

use routes::create_router;

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

    info!("Starting beam-stream...");

    let (router, api) = create_router().split_for_parts();
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
