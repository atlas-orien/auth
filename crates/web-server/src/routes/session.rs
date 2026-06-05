use axum::{Router, routing::post};

use crate::handlers::auth::{login, refresh_token, register};

pub fn session_routes() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh_token", post(refresh_token))
}
