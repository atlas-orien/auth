use axum::http::StatusCode;
use db_core::error::{BIZ_INTERNAL_ERROR, BizError};
use error_code::auth;
use thiserror::Error;
use toolcraft_axum_kit::{ApiError, CommonError};
use validator::{ValidationErrors, ValidationErrorsKind};

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum Error {
    #[error("config error: {0}")]
    #[allow(clippy::enum_variant_names)]
    Config(#[from] toolcraft_config::error::Error),

    #[error("validation error: {0}")]
    #[allow(clippy::enum_variant_names)]
    Validation(#[from] validator::ValidationErrors),

    #[error(transparent)]
    Jwt(#[from] toolcraft_jwt::error::Error), // catch-all for other errors

    #[error("bad request: {message}")]
    BadRequest { code: i16, message: String },

    #[error("custom error: {0}")]
    Custom(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        match err {
            Error::Validation(errors) => {
                if let Some(code) = validation_error_code(&errors) {
                    (
                        StatusCode::BAD_REQUEST,
                        CommonError {
                            code,
                            message: format_validation_errors(&errors),
                        }
                        .to_json(),
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        CommonError {
                            code: BIZ_INTERNAL_ERROR,
                            message: "validation error code is not mapped".to_string(),
                        }
                        .to_json(),
                    )
                }
            }
            Error::BadRequest { code, message } => (
                StatusCode::BAD_REQUEST,
                CommonError { code, message }.to_json(),
            ),
            err => (
                StatusCode::INTERNAL_SERVER_ERROR,
                CommonError {
                    code: BIZ_INTERNAL_ERROR,
                    message: err.to_string(),
                }
                .to_json(),
            ),
        }
    }
}

pub fn bad_request(code: i16, message: impl Into<String>) -> Error {
    Error::BadRequest {
        code,
        message: message.into(),
    }
}

pub fn from_biz_error(err: BizError) -> ApiError {
    (
        StatusCode::BAD_REQUEST,
        CommonError {
            code: err.code(),
            message: err.message().to_string(),
        }
        .to_json(),
    )
}

fn validation_error_code(errors: &ValidationErrors) -> Option<i16> {
    let mut codes = Vec::new();
    collect_validation_codes(errors, &mut codes);
    codes.into_iter().min()
}

fn collect_validation_codes(errors: &ValidationErrors, codes: &mut Vec<i16>) {
    let mut fields = errors.errors().iter().collect::<Vec<_>>();
    fields.sort_by_key(|(left, _)| *left);

    for (_, kind) in fields {
        match kind {
            ValidationErrorsKind::Field(field_errors) => {
                for error in field_errors {
                    if let Some(code) = auth::validation_rule_code(error.code.as_ref()) {
                        codes.push(code);
                    }
                }
            }
            ValidationErrorsKind::Struct(nested) => {
                collect_validation_codes(nested, codes);
            }
            ValidationErrorsKind::List(items) => {
                for nested in items.values() {
                    collect_validation_codes(nested, codes);
                }
            }
        }
    }
}

fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    collect_validation_messages("", errors, &mut messages);

    if messages.is_empty() {
        "Validation failed".to_string()
    } else {
        messages.join("; ")
    }
}

fn collect_validation_messages(
    prefix: &str,
    errors: &ValidationErrors,
    messages: &mut Vec<String>,
) {
    let mut fields = errors.errors().iter().collect::<Vec<_>>();
    fields.sort_by_key(|(left, _)| *left);

    for (field, kind) in fields {
        let path = if prefix.is_empty() {
            field.to_string()
        } else {
            format!("{prefix}.{field}")
        };

        match kind {
            ValidationErrorsKind::Field(field_errors) => {
                for error in field_errors {
                    let message = error
                        .message
                        .as_ref()
                        .map(|message| message.to_string())
                        .unwrap_or_else(|| format!("{} is invalid", error.code));
                    messages.push(format!("{path}: {message}"));
                }
            }
            ValidationErrorsKind::Struct(nested) => {
                collect_validation_messages(&path, nested, messages);
            }
            ValidationErrorsKind::List(items) => {
                for (index, nested) in items {
                    collect_validation_messages(&format!("{path}[{index}]"), nested, messages);
                }
            }
        }
    }
}
