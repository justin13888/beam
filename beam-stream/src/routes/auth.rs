use beam_stream::state::{AppServices, AppState};
use salvo::prelude::*;
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

#[handler]
pub async fn register(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let body: RegisterRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain("Invalid request body"));
            return;
        }
    };

    // TODO: Get device info from headers
    let device_hash = "unknown_device";
    let ip = "0.0.0.0";

    match state
        .services
        .auth
        .register(&body.username, &body.email, &body.password, device_hash, ip)
        .await
    {
        Ok(result) => {
            res.status_code(StatusCode::OK);
            res.render(Json(result));
        }
        Err(err) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain(err.to_string()));
        }
    }
}

#[handler]
pub async fn login(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let body: LoginRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain("Invalid request body"));
            return;
        }
    };

    // TODO: Get device info from headers
    let device_hash = "unknown_device";
    let ip = "0.0.0.0";

    match state
        .services
        .auth
        .login(&body.username_or_email, &body.password, device_hash, ip)
        .await
    {
        Ok(result) => {
            res.status_code(StatusCode::OK);
            res.render(Json(result));
        }
        Err(err) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Text::Plain(err.to_string()));
        }
    }
}

#[handler]
pub async fn refresh(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let body: RefreshRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain("Invalid request body"));
            return;
        }
    };

    match state.services.auth.refresh(&body.session_id).await {
        Ok(result) => {
            res.status_code(StatusCode::OK);
            res.render(Json(result));
        }
        Err(err) => {
            res.status_code(StatusCode::UNAUTHORIZED);
            res.render(Text::Plain(err.to_string()));
        }
    }
}

#[handler]
pub async fn logout(req: &mut Request, depot: &mut Depot, res: &mut Response) {
    let state = depot.obtain::<AppState>().unwrap();
    let body: RefreshRequest = match req.parse_json().await {
        Ok(b) => b,
        Err(_) => {
            res.status_code(StatusCode::BAD_REQUEST);
            res.render(Text::Plain("Invalid request body"));
            return;
        }
    };

    match state.services.auth.logout(&body.session_id).await {
        Ok(_) => {
            res.status_code(StatusCode::OK);
        }
        Err(err) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain(err.to_string()));
        }
    }
}

pub fn auth_routes() -> Router {
    Router::new()
        .push(Router::with_path("register").post(register))
        .push(Router::with_path("login").post(login))
        .push(Router::with_path("refresh").post(refresh))
        .push(Router::with_path("logout").post(logout))
}
