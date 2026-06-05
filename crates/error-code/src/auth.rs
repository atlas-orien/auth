// Auth route business error codes
pub const USERNAME_ALREADY_EXISTS: i16 = -10000;
pub const EMAIL_ALREADY_EXISTS: i16 = -10001;
pub const USER_NOT_FOUND: i16 = -10002;
pub const USER_DISABLED: i16 = -10003;
pub const PASSWORD_HASH_FAILED: i16 = -10004;
pub const PASSWORD_HASH_PARSE_FAILED: i16 = -10005;
pub const PASSWORD_INVALID: i16 = -10006;
pub const USERNAME_LENGTH_INVALID: i16 = -10007;
pub const USERNAME_FORMAT_INVALID: i16 = -10008;
pub const DISPLAY_NAME_LENGTH_INVALID: i16 = -10009;
pub const AVATAR_URL_INVALID: i16 = -10010;
pub const REGISTER_PASSWORD_LENGTH_INVALID: i16 = -10011;
pub const EMAIL_INVALID: i16 = -10012;
pub const IDENTIFIER_LENGTH_INVALID: i16 = -10013;
pub const OLD_PASSWORD_LENGTH_INVALID: i16 = -10014;
pub const NEW_PASSWORD_LENGTH_INVALID: i16 = -10015;
pub const AVATAR_KEY_LENGTH_INVALID: i16 = -10016;
pub const AVATAR_KEY_FORMAT_INVALID: i16 = -10017;
pub const AVATAR_KEY_URL_NOT_ALLOWED: i16 = -10018;
pub const LOGIN_PASSWORD_LENGTH_INVALID: i16 = -10019;

pub const USERNAME_LENGTH_INVALID_RULE: &str = "username_length_invalid";
pub const USERNAME_FORMAT_INVALID_RULE: &str = "username_format_invalid";
pub const DISPLAY_NAME_LENGTH_INVALID_RULE: &str = "display_name_length_invalid";
pub const AVATAR_URL_INVALID_RULE: &str = "avatar_url_invalid";
pub const REGISTER_PASSWORD_LENGTH_INVALID_RULE: &str = "register_password_length_invalid";
pub const EMAIL_INVALID_RULE: &str = "email_invalid";
pub const IDENTIFIER_LENGTH_INVALID_RULE: &str = "identifier_length_invalid";
pub const LOGIN_PASSWORD_LENGTH_INVALID_RULE: &str = "login_password_length_invalid";
pub const OLD_PASSWORD_LENGTH_INVALID_RULE: &str = "old_password_length_invalid";
pub const NEW_PASSWORD_LENGTH_INVALID_RULE: &str = "new_password_length_invalid";

pub fn validation_rule_code(rule: &str) -> Option<i16> {
    match rule {
        USERNAME_LENGTH_INVALID_RULE => Some(USERNAME_LENGTH_INVALID),
        USERNAME_FORMAT_INVALID_RULE => Some(USERNAME_FORMAT_INVALID),
        DISPLAY_NAME_LENGTH_INVALID_RULE => Some(DISPLAY_NAME_LENGTH_INVALID),
        AVATAR_URL_INVALID_RULE => Some(AVATAR_URL_INVALID),
        REGISTER_PASSWORD_LENGTH_INVALID_RULE => Some(REGISTER_PASSWORD_LENGTH_INVALID),
        EMAIL_INVALID_RULE => Some(EMAIL_INVALID),
        IDENTIFIER_LENGTH_INVALID_RULE => Some(IDENTIFIER_LENGTH_INVALID),
        LOGIN_PASSWORD_LENGTH_INVALID_RULE => Some(LOGIN_PASSWORD_LENGTH_INVALID),
        OLD_PASSWORD_LENGTH_INVALID_RULE => Some(OLD_PASSWORD_LENGTH_INVALID),
        NEW_PASSWORD_LENGTH_INVALID_RULE => Some(NEW_PASSWORD_LENGTH_INVALID),
        _ => None,
    }
}

pub fn user_message(code: i16) -> Option<&'static str> {
    match code {
        USERNAME_ALREADY_EXISTS => Some("username already exists"),
        EMAIL_ALREADY_EXISTS => Some("email already exists"),
        USER_NOT_FOUND => Some("user not found"),
        USER_DISABLED => Some("user account is disabled"),
        PASSWORD_HASH_FAILED => Some("password hash failed"),
        PASSWORD_HASH_PARSE_FAILED => Some("password hash parse failed"),
        PASSWORD_INVALID => Some("password is invalid"),
        USERNAME_LENGTH_INVALID => Some("username length must be between 3 and 32"),
        USERNAME_FORMAT_INVALID => {
            Some("username may only contain letters, numbers, underscores, and hyphens")
        }
        DISPLAY_NAME_LENGTH_INVALID => Some("display_name length must be between 1 and 64"),
        AVATAR_URL_INVALID => Some("avatar must be a valid URL"),
        REGISTER_PASSWORD_LENGTH_INVALID => Some("password length must be between 8 and 128"),
        EMAIL_INVALID => Some("email must be a valid email address"),
        IDENTIFIER_LENGTH_INVALID => Some("identifier length must be between 1 and 128"),
        OLD_PASSWORD_LENGTH_INVALID => Some("oldPassword length must be between 1 and 128"),
        NEW_PASSWORD_LENGTH_INVALID => Some("newPassword length must be between 8 and 128"),
        AVATAR_KEY_LENGTH_INVALID => Some("avatar key length must be between 1 and 512"),
        AVATAR_KEY_FORMAT_INVALID => Some("avatar key format is invalid"),
        AVATAR_KEY_URL_NOT_ALLOWED => Some("avatar must be object key only (URL is not allowed)"),
        LOGIN_PASSWORD_LENGTH_INVALID => Some("password length must be between 1 and 128"),
        _ => None,
    }
}

pub fn validation_rule_user_message(rule: &str) -> Option<&'static str> {
    validation_rule_code(rule).and_then(user_message)
}
