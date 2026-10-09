mod dto;
mod error;
mod handlers;
mod logging;
mod routes;
mod settings;
mod statics;

use std::sync::Arc;

use settings::Settings;
use toolcraft_axum_kit::http_server;
use toolcraft_jwt::Jwt;

use crate::{
    logging::init_tracing_to_file,
    statics::{db_manager::init_db, internal_auth::init_internal_auth_config},
};

#[tokio::main]
async fn main() {
    init_tracing_to_file();
    let settings = Settings::load("config/services.toml").unwrap();
    let jwt_verify_cfg = Arc::new(settings.build_jwt_verify_config().unwrap());
    let internal_auth_cfg = settings.build_internal_auth_config().unwrap();
    init_internal_auth_config(internal_auth_cfg);
    init_db(settings.db.clone())
        .await
        .expect("DatabaseManager initialization failed");

    let jwt = Arc::new(Jwt::new(settings.jwt));
    let router = routes::create_routes(jwt, jwt_verify_cfg);
    let http_task = http_server::start(settings.http.port, router);

    let _ = tokio::join!(http_task);
}
