use std::sync::atomic::Ordering;

use async_graphql::http::GraphiQLSource;
use eyre::{Result, eyre};
use salvo::cors::Cors;
use salvo::prelude::*;
use tracing::info;

use beam_stream::graphql::{AppSchema, SharedAppState, UserContext, create_schema};
use beam_stream::{config::Config, graphql::AppContext};
use routes::create_router;

mod routes;

const GRAPHQL_PATH: &str = "graphql";

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

    // Connect to Database
    info!("Connecting to database at {}", config.database_url);
    let db = sea_orm::Database::connect(&config.database_url)
        .await
        .map_err(|e| eyre!("Failed to connect to database: {}", e))?;
    info!("Connected to database");

    let (schema, state) = create_schema(&config, db).await;

    // Build CORS handler
    let cors = Cors::new()
        .allow_origin(salvo::cors::Any)
        .allow_methods(salvo::cors::Any)
        .allow_headers(salvo::cors::Any)
        .into_handler();

    // Build API router
    let api_router = create_router();

    // Build auth router
    let auth_router = Router::with_path("auth").push(routes::auth::auth_routes());

    // Build GraphQL router
    let graphql_router = Router::with_path(GRAPHQL_PATH)
        .get(graphiql)
        .post(graphql_handler);

    // Combine all routers
    let router = Router::new()
        .hoop(cors)
        .hoop(affix_state::inject(schema).inject(state))
        .push(api_router)
        .push(auth_router)
        .push(graphql_router);

    // Generate OpenAPI documentation
    let doc = OpenApi::new("Beam Stream API", "1.0.0").merge_router(&router);
    let router = router
        .push(doc.into_router("/api-doc/openapi.json"))
        .push(Scalar::new("/api-doc/openapi.json").into_router("/openapi"));

    info!("Binding to address: {}", config.bind_address);
    let acceptor = TcpListener::new(&config.bind_address).bind().await;

    info!("Server listening on {}", config.bind_address);
    info!(
        "API documentation available at http://{}/openapi",
        config.bind_address
    );
    info!(
        "GraphiQL interface available at http://{}/{}",
        config.bind_address, GRAPHQL_PATH
    );

    Server::new(acceptor).serve(router).await;

    Ok(())
}

#[handler]
async fn graphiql(res: &mut Response) {
    res.render(Text::Html(
        GraphiQLSource::build()
            .endpoint(&format!("/{}", GRAPHQL_PATH))
            .finish(),
    ));
}

#[handler]
async fn graphql_handler(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let schema = depot.obtain::<AppSchema>().unwrap().clone();
    let state = depot.obtain::<SharedAppState>().unwrap().clone();

    // Parse GraphQL request from body
    let body_bytes = match req.payload().await {
        Ok(bytes) => bytes.to_vec(),
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain("Failed to read request body"));
            return;
        }
    };

    let gql_request: async_graphql::Request = match serde_json::from_slice(&body_bytes) {
        Ok(r) => r,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain("Invalid GraphQL request"));
            return;
        }
    };

    let mut gql_request = gql_request;
    let mut user_context = None;

    if let Some(auth_header) = req.headers().get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
        && auth_str.starts_with("Bearer ")
    {
        let token = &auth_str[7..];

        // Get AuthService from state
        match state.services.auth.verify_token(token).await {
            Ok(user) => {
                user_context = Some(UserContext {
                    user_id: user.user_id,
                });
            }
            Err(e) => {
                tracing::warn!("Failed to verify token: {}", e);
            }
        }
    }

    let app_context = AppContext::new(user_context);
    gql_request = gql_request.data(app_context);

    let gql_response = schema.execute(gql_request).await;

    // Serialize response
    match serde_json::to_vec(&gql_response) {
        Ok(json_bytes) => {
            res.headers_mut()
                .insert("Content-Type", "application/json".parse().unwrap());
            res.body(salvo::http::body::ResBody::Once(bytes::Bytes::from(
                json_bytes,
            )));
        }
        Err(_) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain("Failed to serialize GraphQL response"));
        }
    }
}
