use std::sync::Arc;

use axum::{Extension, Json};
use service::api::auth::AuthApi;
use toolcraft_axum_kit::{ApiError, Empty, IntoCommonResponse, ResponseResult};
use toolcraft_jwt::Jwt;
use validator::Validate;

use crate::{
    dto::auth::{
        LoginRequest, LoginResponse, RefreshTokenRequest, RefreshTokenResponse, RegisterRequest,
    },
    error::{Error, from_biz_error},
    handlers::user::validate_avatar_key,
    statics::db_manager::get_default_ctx,
};

pub async fn register(Json(req): Json<RegisterRequest>) -> ResponseResult<Empty> {
    req.validate()
        .map_err(Error::Validation)
        .map_err(ApiError::from)?;
    if let Some(avatar_key) = req.avatar.as_deref() {
        validate_avatar_key(avatar_key).map_err(ApiError::from)?;
    }

    let api = AuthApi::new(get_default_ctx());
    api.register(service::dto::auth::RegisterRequest {
        username: req.username,
        display_name: req.display_name,
        avatar: req.avatar,
        password: req.password,
        email: req.email,
    })
    .await
    .map_err(from_biz_error)?;

    Ok(Empty.into_common_response().to_json())
}

pub async fn login(
    Extension(jwt): Extension<Arc<Jwt>>,
    Json(req): Json<LoginRequest>,
) -> ResponseResult<LoginResponse> {
    req.validate()
        .map_err(Error::Validation)
        .map_err(ApiError::from)?;

    let api = AuthApi::new(get_default_ctx());
    let user = api
        .login(service::dto::auth::LoginRequest {
            identifier: req.identifier,
            password: req.password,
        })
        .await
        .map_err(from_biz_error)?;

    let token_pair = jwt
        .generate_token_pair_for_subject(user.id)
        .map_err(Error::from)?;

    Ok(LoginResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
    }
    .into_common_response()
    .to_json())
}

pub async fn refresh_token(
    Extension(jwt): Extension<Arc<Jwt>>,
    Json(req): Json<RefreshTokenRequest>,
) -> ResponseResult<RefreshTokenResponse> {
    let access_token = jwt
        .refresh_access_token(&req.refresh_token)
        .map_err(Error::from)?;

    Ok(RefreshTokenResponse { access_token }
        .into_common_response()
        .to_json())
}
