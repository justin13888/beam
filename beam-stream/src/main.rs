use std::net::{Ipv4Addr, SocketAddr};

use axum::{middleware, Router};
use dotenvy::dotenv;
use http::Method;
use listenfd::ListenFd;
use metrics::start_metrics_server;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::{Config, Environment},
    metrics::track_metrics,
    task::task_router,
};

mod config;
mod metrics;
mod task;

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
        start_main_server(&config.binding_address).await;
    }

    Ok(())
}

async fn start_main_server(address: &SocketAddr) {
    // Define the OpenAPI documentation
    #[derive(OpenApi)]
    #[openapi(
        paths(
            task::list_tasks,
            task::get_task,
            task::search_tasks,
            task::create_task,
            task::delete_task,
        ),
        components(
            schemas(
                task::Task,
                task::TaskType,
                task::TaskTrigger,
                task::TaskStatus,
                task::ScanType,
                task::CollectionScanTask,
                task::TaskError,
                task::TaskSearchQuery,
                task::CreateTask,
                task::CreateCollectionScanTask,
            )
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "task", description = "Task management API")
        )
    )]
    struct ApiDoc;

    // Define a security addon to add API key security scheme
    // TODO: Modify for JWT
    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("task_apikey"))),
                )
            }
        }
    }
    // TODO: implement JWT on all routes

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);

    // TODO: Implement versioning
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc"))
        .merge(task_router())
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
