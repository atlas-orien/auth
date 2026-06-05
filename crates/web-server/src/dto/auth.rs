use serde::{Deserialize, Deserializer, Serialize};
use validator::{Validate, ValidationError};

// ── Requests ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(
        length(
            min = 3,
            max = 32,
            code = "username_length_invalid",
            message = "username length must be between 3 and 32"
        ),
        custom(
            function = "validate_username",
            code = "username_format_invalid",
            message = "username may only contain letters, numbers, underscores, and hyphens"
        )
    )]
    pub username: String,
    #[validate(length(
        min = 1,
        max = 64,
        code = "display_name_length_invalid",
        message = "display_name length must be between 1 and 64"
    ))]
    pub display_name: Option<String>,
    #[validate(url(code = "avatar_url_invalid", message = "avatar must be a valid URL"))]
    pub avatar: Option<String>,
    #[validate(length(
        min = 8,
        max = 128,
        code = "password_length_invalid",
        message = "password length must be between 8 and 128"
    ))]
    pub password: String,
    #[validate(email(
        code = "email_invalid",
        message = "email must be a valid email address"
    ))]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(
        min = 1,
        max = 128,
        code = "identifier_length_invalid",
        message = "identifier length must be between 1 and 128"
    ))]
    pub identifier: String,
    #[validate(length(
        min = 1,
        max = 128,
        code = "password_length_invalid",
        message = "password length must be between 1 and 128"
    ))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePasswordRequest {
    #[serde(rename = "oldPassword")]
    #[validate(length(
        min = 1,
        max = 128,
        code = "old_password_length_invalid",
        message = "oldPassword length must be between 1 and 128"
    ))]
    pub old_password: String,
    #[serde(rename = "newPassword")]
    #[validate(length(
        min = 8,
        max = 128,
        code = "new_password_length_invalid",
        message = "newPassword length must be between 8 and 128"
    ))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEmailRequest {
    #[validate(email(
        code = "email_invalid",
        message = "email must be a valid email address"
    ))]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub display_name: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_nullable_string")]
    pub avatar: Option<Option<String>>,
}

fn deserialize_optional_nullable_string<'de, D>(
    deserializer: D,
) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        Ok(())
    } else {
        Err(ValidationError::new("username"))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::UpdateProfileRequest;

    #[test]
    fn update_profile_request_distinguishes_missing_null_and_value() {
        let missing: UpdateProfileRequest = serde_json::from_value(json!({})).unwrap();
        assert_eq!(missing.avatar, None);

        let cleared: UpdateProfileRequest =
            serde_json::from_value(json!({ "avatar": null })).unwrap();
        assert_eq!(cleared.avatar, Some(None));

        let updated: UpdateProfileRequest =
            serde_json::from_value(json!({ "avatar": "avatars/user.png" })).unwrap();
        assert_eq!(updated.avatar, Some(Some("avatars/user.png".to_string())));
    }
}

// ── Responses ────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub display_user_id: Option<String>,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub email: Option<String>,
    pub email_verified: bool,
    pub disabled: bool,
}
