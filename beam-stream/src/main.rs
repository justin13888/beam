use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{middleware, Router};
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

use crate::{metrics::track_metrics, todo::todo_router};

mod metrics;
mod todo;

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "beam_stream=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init(); // TODO: implement based on env

    let metrics_address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8081));

    let (_main_server, _metrics_server) =
        tokio::join!(start_main_server(), start_metrics_server(&metrics_address));
}

async fn start_main_server() {
    // Define the OpenAPI documentation
    #[derive(OpenApi)]
    #[openapi(
        paths(
            todo::list_todos,
            todo::search_todos,
            todo::create_todo,
            todo::mark_done,
            todo::delete_todo,
        ),
        components(
            schemas(todo::Todo, todo::TodoError)
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    // Define a security addon to add API key security scheme
    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
                )
            }
        }
    }

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
        .merge(todo_router())
        .layer(cors)
        .route_layer(middleware::from_fn(track_metrics));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));

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
