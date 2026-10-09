use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::handlers::internal::{
    display_user_id_to_uuid, internal_auth, jwt_verify_config, register_avatar_provider,
    user_by_identifier,
};

pub fn internal_routes() -> Router {
    Router::new()
        .route("/jwt_verify_config", get(jwt_verify_config))
        .route("/avatar_provider", post(register_avatar_provider))
        .route("/users/by_identifier", get(user_by_identifier))
        .route(
            "/users/{display_user_id}/uuid",
            get(display_user_id_to_uuid),
        )
        .route_layer(middleware::from_fn(internal_auth))
}
