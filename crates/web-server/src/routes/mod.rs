mod internal;
mod session;
mod user;

use std::sync::Arc;

use axum::{Extension, Router};
use internal::internal_routes;
use session::session_routes;
use toolcraft_axum_kit::middleware::cors::create_cors;
use toolcraft_jwt::Jwt;
use user::user_routes;

pub fn create_routes(
    jwt: Arc<Jwt>,
    jwt_verify_cfg: Arc<crate::settings::JwtVerifyConfig>,
    avatar_url_cfg: Arc<crate::settings::AvatarUrlConfig>,
) -> Router {
    let cors = create_cors();

    let auth_routes = Router::new()
        .nest("/session", session_routes())
        .nest("/user", user_routes());

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/internal", internal_routes())
        .layer(Extension(jwt))
        .layer(Extension(jwt_verify_cfg))
        .layer(Extension(avatar_url_cfg))
        .layer(cors)
}
