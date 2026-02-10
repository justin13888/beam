use axum::{
    Router,
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use beam_stream::graphql::AppServices;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub session_id: String,
}

pub async fn register(
    Extension(services): Extension<Arc<AppServices>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // TODO: Get device info from headers
    let device_hash = "unknown_device";
    let ip = "0.0.0.0";

    match services
        .auth
        .register(&req.username, &req.email, &req.password, device_hash, ip)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, err.to_string()).into_response(),
    }
}

pub async fn login(
    Extension(services): Extension<Arc<AppServices>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    // TODO: Get device info from headers
    let device_hash = "unknown_device";
    let ip = "0.0.0.0";

    match services
        .auth
        .login(&req.username_or_email, &req.password, device_hash, ip)
        .await
    {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(err) => (StatusCode::UNAUTHORIZED, err.to_string()).into_response(),
    }
}

pub async fn refresh(
    Extension(services): Extension<Arc<AppServices>>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    match services.auth.refresh(&req.session_id).await {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(err) => (StatusCode::UNAUTHORIZED, err.to_string()).into_response(),
    }
}

pub async fn logout(
    Extension(services): Extension<Arc<AppServices>>,
    Json(req): Json<RefreshRequest>, // Re-using RefreshRequest since it has session_id
) -> impl IntoResponse {
    match services.auth.logout(&req.session_id).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub fn auth_routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}
