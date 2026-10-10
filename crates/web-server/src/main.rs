mod dto;
mod error;
mod handlers;
mod logging;
mod routes;
mod settings;
mod statics;

use std::{process::ExitCode, sync::Arc};

use settings::Settings;
use toolcraft_axum_kit::http_server;
use toolcraft_jwt::Jwt;
use tracing::error;

use crate::{
    logging::init_tracing_to_file,
    statics::{db_manager::init_db, internal_auth::init_internal_auth_config},
};

#[tokio::main]
async fn main() -> ExitCode {
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
    let port = settings.http.port;

    match http_server::start(port, router).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!(port, error = %err, "HTTP server failed");
            eprintln!("HTTP server failed on 0.0.0.0:{port}: {err}");
            ExitCode::FAILURE
        }
    }
}
