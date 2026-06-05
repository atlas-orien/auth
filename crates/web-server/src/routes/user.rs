use axum::{
    Router, middleware,
    routing::{get, post, put},
};
use toolcraft_axum_kit::middleware::auth_mw;
use toolcraft_jwt::Jwt;

use crate::handlers::user::{me, update_email, update_password, update_profile, verify_email};

pub fn user_routes() -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/password", put(update_password))
        .route("/email", put(update_email))
        .route("/profile", put(update_profile))
        .route("/email/verify", post(verify_email))
        .route_layer(middleware::from_fn(auth_mw::auth::<Jwt>))
}
