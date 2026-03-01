#[cfg(test)]
#[path = "routes_tests.rs"]
mod routes_tests;

use crate::server::error::ErrorBody;
use crate::utils::service::AuthService;
use salvo::oapi::{ToResponses, ToSchema};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

fn device_hash_from_request(req: &Request) -> String {
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    format!("{:x}", Sha256::digest(user_agent.as_bytes()))
}

fn extract_client_ip(req: &Request) -> String {
    if let Some(forwarded_for) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        && let Some(first) = forwarded_for.split(',').next()
    {
        return first.trim().to_string();
    }
    if let Some(real_ip) = req.headers().get("x-real-ip").and_then(|v| v.to_str().ok()) {
        return real_ip.to_string();
    }
    "unknown".to_string()
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub session_id: String,
}

// ── Error enums ───────────────────────────────────────────────────────────────

#[derive(ToResponses)]
pub enum RegisterError {
    /// Invalid request body or user already exists
    #[salvo(response(status_code = 400))]
    BadRequest(ErrorBody),
}

#[async_trait]
impl Writer for RegisterError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::BadRequest(body) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(body));
            }
        }
    }
}

#[derive(ToResponses)]
pub enum LoginError {
    /// Invalid request body
    #[salvo(response(status_code = 400))]
    BadRequest(ErrorBody),
    /// Invalid credentials
    #[salvo(response(status_code = 401))]
    Unauthorized(ErrorBody),
}

#[async_trait]
impl Writer for LoginError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::BadRequest(body) => {
                res.status_code(StatusCode::BAD_REQUEST);
                res.render(Json(body));
            }
            Self::Unauthorized(body) => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(body));
            }
        }
    }
}

#[derive(ToResponses)]
pub enum RefreshError {
    /// Missing or invalid session
    #[salvo(response(status_code = 401))]
    Unauthorized(ErrorBody),
}

#[async_trait]
impl Writer for RefreshError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::Unauthorized(body) => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(body));
            }
        }
    }
}

#[derive(ToResponses)]
pub enum LogoutError {
    /// Internal server error
    #[salvo(response(status_code = 500))]
    InternalError(ErrorBody),
}

#[async_trait]
impl Writer for LogoutError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::InternalError(body) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(body));
            }
        }
    }
}

#[derive(ToResponses)]
pub enum LogoutAllError {
    /// Invalid or missing JWT
    #[salvo(response(status_code = 401))]
    Unauthorized(ErrorBody),
    /// Internal server error
    #[salvo(response(status_code = 500))]
    InternalError(ErrorBody),
}

#[async_trait]
impl Writer for LogoutAllError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::Unauthorized(body) => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(body));
            }
            Self::InternalError(body) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(body));
            }
        }
    }
}

#[derive(ToResponses)]
pub enum ListSessionsError {
    /// Invalid or missing JWT
    #[salvo(response(status_code = 401))]
    Unauthorized(ErrorBody),
    /// Internal server error
    #[salvo(response(status_code = 500))]
    InternalError(ErrorBody),
}

#[async_trait]
impl Writer for ListSessionsError {
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        match self {
            Self::Unauthorized(body) => {
                res.status_code(StatusCode::UNAUTHORIZED);
                res.render(Json(body));
            }
            Self::InternalError(body) => {
                res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
                res.render(Json(body));
            }
        }
    }
}

// ── Endpoints ─────────────────────────────────────────────────────────────────

/// Register a new user account
#[endpoint(
    tags("auth"),
    request_body = RegisterRequest,
)]
pub async fn register(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<Json<crate::utils::service::AuthResponse>, RegisterError> {
    let auth = depot.obtain::<Arc<dyn AuthService>>().unwrap().clone();
    let body: RegisterRequest = req.parse_json().await.map_err(|_| {
        RegisterError::BadRequest(ErrorBody::new("invalid_request", "Invalid request body"))
    })?;

    let device_hash = device_hash_from_request(req);
    let ip = extract_client_ip(req);

    let auth_response = auth
        .register(
            &body.username,
            &body.email,
            &body.password,
            &device_hash,
            &ip,
        )
        .await
        .map_err(|err| {
            RegisterError::BadRequest(ErrorBody::new("user_already_exists", err.to_string()))
        })?;

    let cookie =
        salvo::http::cookie::Cookie::build(("session_id", auth_response.session_id.clone()))
            .path("/")
            .http_only(true)
            .same_site(salvo::http::cookie::SameSite::Lax)
            .max_age(salvo::http::cookie::time::Duration::days(7))
            .build();
    res.add_cookie(cookie);

    Ok(Json(auth_response))
}

/// Login with username/email and password
#[endpoint(
    tags("auth"),
    request_body = LoginRequest,
)]
pub async fn login(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<Json<crate::utils::service::AuthResponse>, LoginError> {
    let auth = depot.obtain::<Arc<dyn AuthService>>().unwrap().clone();
    let body: LoginRequest = req.parse_json().await.map_err(|_| {
        LoginError::BadRequest(ErrorBody::new("invalid_request", "Invalid request body"))
    })?;

    let device_hash = device_hash_from_request(req);
    let ip = extract_client_ip(req);

    let auth_response = auth
        .login(&body.username_or_email, &body.password, &device_hash, &ip)
        .await
        .map_err(|_| {
            LoginError::Unauthorized(ErrorBody::new(
                "invalid_credentials",
                "Invalid username or password",
            ))
        })?;

    let cookie =
        salvo::http::cookie::Cookie::build(("session_id", auth_response.session_id.clone()))
            .path("/")
            .http_only(true)
            .same_site(salvo::http::cookie::SameSite::Lax)
            .max_age(salvo::http::cookie::time::Duration::days(7))
            .build();
    res.add_cookie(cookie);

    Ok(Json(auth_response))
}

/// Refresh an existing session using a session cookie or request body
#[endpoint(
    tags("auth"),
    request_body(content = RefreshRequest, description = "Session ID (alternative to session cookie)"),
)]
pub async fn refresh(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<Json<crate::utils::service::AuthResponse>, RefreshError> {
    let auth = depot.obtain::<Arc<dyn AuthService>>().unwrap().clone();

    let session_id = if let Some(c) = req.cookie("session_id") {
        c.value().to_string()
    } else if let Ok(body) = req.parse_json::<RefreshRequest>().await {
        body.session_id
    } else {
        return Err(RefreshError::Unauthorized(ErrorBody::new(
            "unauthorized",
            "Missing session cookie or body",
        )));
    };

    let auth_response = auth.refresh(&session_id).await.map_err(|_| {
        RefreshError::Unauthorized(ErrorBody::new(
            "session_not_found",
            "Invalid or expired session",
        ))
    })?;

    let cookie =
        salvo::http::cookie::Cookie::build(("session_id", auth_response.session_id.clone()))
            .path("/")
            .http_only(true)
            .same_site(salvo::http::cookie::SameSite::Lax)
            .max_age(salvo::http::cookie::time::Duration::days(7))
            .build();
    res.add_cookie(cookie);

    Ok(Json(auth_response))
}

/// Logout and revoke the current session
#[endpoint(tags("auth"))]
pub async fn logout(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), LogoutError> {
    let auth = depot.obtain::<Arc<dyn AuthService>>().unwrap().clone();

    let session_id = if let Some(c) = req.cookie("session_id") {
        c.value().to_string()
    } else if let Ok(body) = req.parse_json::<RefreshRequest>().await {
        body.session_id
    } else {
        // Already logged out or no session — idempotent 200
        return Ok(());
    };

    // Remove cookie
    res.remove_cookie("session_id");

    auth.logout(&session_id).await.map_err(|err| {
        LogoutError::InternalError(ErrorBody::new("internal_error", err.to_string()))
    })?;

    Ok(())
}

#[derive(Serialize, ToSchema)]
pub struct LogoutAllResponse {
    /// Number of sessions that were revoked
    pub revoked: u64,
}

#[derive(Serialize, ToSchema)]
pub struct SessionSummary {
    pub session_id: String,
    pub device_hash: String,
    pub ip: String,
    pub created_at: i64,
    pub last_active: i64,
}

fn extract_bearer_token(req: &Request) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .filter(|s| s.starts_with("Bearer "))
        .map(|s| s[7..].to_string())
}

/// Logout all active sessions for the current user
#[endpoint(tags("auth"))]
pub async fn logout_all(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<LogoutAllResponse>, LogoutAllError> {
    let auth = depot.obtain::<Arc<dyn AuthService>>().unwrap().clone();

    let token = extract_bearer_token(req).ok_or_else(|| {
        LogoutAllError::Unauthorized(ErrorBody::new("unauthorized", "Missing or invalid JWT"))
    })?;

    let user = auth.verify_token(&token).await.map_err(|_| {
        LogoutAllError::Unauthorized(ErrorBody::new("unauthorized", "Invalid or expired token"))
    })?;

    let revoked = auth.logout_all(&user.user_id).await.map_err(|err| {
        LogoutAllError::InternalError(ErrorBody::new("internal_error", err.to_string()))
    })?;

    Ok(Json(LogoutAllResponse { revoked }))
}

/// List all active sessions for the current user
#[endpoint(tags("auth"))]
pub async fn list_sessions(
    req: &mut Request,
    depot: &mut Depot,
) -> Result<Json<Vec<SessionSummary>>, ListSessionsError> {
    let auth = depot.obtain::<Arc<dyn AuthService>>().unwrap().clone();

    let token = extract_bearer_token(req).ok_or_else(|| {
        ListSessionsError::Unauthorized(ErrorBody::new("unauthorized", "Missing or invalid JWT"))
    })?;

    let user = auth.verify_token(&token).await.map_err(|_| {
        ListSessionsError::Unauthorized(ErrorBody::new("unauthorized", "Invalid or expired token"))
    })?;

    let sessions = auth.get_sessions(&user.user_id).await.map_err(|err| {
        ListSessionsError::InternalError(ErrorBody::new("internal_error", err.to_string()))
    })?;

    let summaries: Vec<SessionSummary> = sessions
        .into_iter()
        .map(|(session_id, data)| SessionSummary {
            session_id,
            device_hash: data.device_hash,
            ip: data.ip,
            created_at: data.created_at,
            last_active: data.last_active,
        })
        .collect();

    Ok(Json(summaries))
}

pub fn auth_routes() -> Router {
    Router::new()
        .push(Router::with_path("register").post(register))
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("refresh").post(refresh))
        .push(Router::with_path("logout").post(logout))
        .push(Router::with_path("logout-all").post(logout_all))
        .push(Router::with_path("sessions").get(list_sessions))
}
