use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use service::api::auth::AuthApi;
use toolcraft_axum_kit::{IntoCommonResponse, ResponseResult};

use crate::{
    dto::{
        auth::UserResponse,
        internal::{
            DisplayUserIdToUuidResponse, GetUserByIdentifierQuery, JwtVerifyConfigResponse,
            RegisterAvatarProviderRequest,
        },
    },
    error::{Error, from_biz_error},
    handlers::user::build_user_response,
    settings::JwtVerifyConfig,
    statics::{
        avatar_provider::register_avatar_provider as set_avatar_provider,
        db_manager::get_default_ctx, internal_auth::get_internal_auth_config,
    },
};

pub async fn jwt_verify_config(
    Extension(cfg): Extension<Arc<JwtVerifyConfig>>,
) -> ResponseResult<JwtVerifyConfigResponse> {
    Ok(JwtVerifyConfigResponse {
        public_key_pem: cfg.public_key_pem.clone(),
        issuer: cfg.issuer.clone(),
        audience: cfg.audience.clone(),
    }
    .into_common_response()
    .to_json())
}

pub async fn display_user_id_to_uuid(
    Path(display_user_id): Path<String>,
) -> ResponseResult<DisplayUserIdToUuidResponse> {
    let api = AuthApi::new(get_default_ctx());
    let user = api
        .get_user_by_display_user_id(&display_user_id)
        .await
        .map_err(from_biz_error)?;

    Ok(DisplayUserIdToUuidResponse { id: user.id }
        .into_common_response()
        .to_json())
}

pub async fn user_by_identifier(
    Query(query): Query<GetUserByIdentifierQuery>,
) -> ResponseResult<UserResponse> {
    let api = AuthApi::new(get_default_ctx());
    let user = api
        .get_user_by_identifier(&query.identifier)
        .await
        .map_err(from_biz_error)?;

    Ok(build_user_response(user).into_common_response().to_json())
}

pub async fn register_avatar_provider(
    axum::Json(req): axum::Json<RegisterAvatarProviderRequest>,
) -> ResponseResult<toolcraft_axum_kit::Empty> {
    let public_base_url = normalize_public_base_url(&req.public_base_url).map_err(Error::Custom)?;
    set_avatar_provider(public_base_url);
    Ok(toolcraft_axum_kit::Empty.into_common_response().to_json())
}

fn normalize_public_base_url(value: &str) -> Result<String, String> {
    let value = value.trim().trim_end_matches('/');
    let url = url::Url::parse(value).map_err(|e| format!("invalid avatar provider URL: {e}"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err("avatar provider URL must use http or https and include a host".to_string());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("avatar provider URL must not include query or fragment".to_string());
    }
    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_public_base_url;

    #[test]
    fn normalizes_avatar_provider_base_url() {
        assert_eq!(
            normalize_public_base_url(" https://files.example.com/ ").unwrap(),
            "https://files.example.com"
        );
    }

    #[test]
    fn rejects_non_http_or_query_avatar_provider_urls() {
        assert!(normalize_public_base_url("ftp://files.example.com").is_err());
        assert!(normalize_public_base_url("https://files.example.com?token=x").is_err());
    }
}

pub async fn internal_auth(req: Request<axum::body::Body>, next: Next) -> Response {
    let cfg = get_internal_auth_config();
    let token = req
        .headers()
        .get(&cfg.header_name)
        .and_then(|v| v.to_str().ok());

    if token != Some(cfg.token.as_str()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    next.run(req).await
}
