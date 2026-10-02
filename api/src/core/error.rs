use std::collections::HashMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("There was an internal server error")]
    InternalServerError(#[from] anyhow::Error),

    #[error("Not found")]
    NotFound,

    #[error("You are not authorized to access this resource")]
    Forbidden,

    #[error("You are not logged in")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("{0}")]
    BadRequest(String),

    #[error("The request body is invalid")]
    ValidationError(std::collections::HashMap<String, Vec<String>>),
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::InternalServerError(_) => "INTERNAL_SERVER",
            AppError::NotFound => "NOT_FOUND",
            AppError::Forbidden => "FORBIDDEN",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::InvalidCredentials => "INVALID_CREDENTIALS",
            AppError::TokenExpired => "TOKEN_EXPIRED",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::ValidationError(_) => "VALIDATION",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::InternalServerError(ref err) => {
                tracing::error!("Internal server error: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            }
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            AppError::TokenExpired => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) | AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
        };

        let body = match &self {
            AppError::ValidationError(details) => {
                // Crate `validator` key lỗi bằng tên field Rust (snake_case), còn contract
                // API camelCase, nên chuyển key về camelCase trước khi trả response.
                let details: HashMap<String, Vec<String>> = details
                    .iter()
                    .map(|(k, v)| (snake_path_to_camel_case(k), v.clone()))
                    .collect();
                Json(json!({
                    "code": self.error_code(),
                    "message": self.to_string(),
                    "details": details
                }))
            }
            _ => Json(json!({
                "code": self.error_code(),
                "message": self.to_string()
            })),
        };

        (status, body).into_response()
    }
}

/// Chuyển key kiểu đường dẫn field (có thể lồng nhau bằng `.`) từ snake_case
/// sang camelCase. VD: `first_name` → `firstName`, `a_b.c_d` → `aB.cD`.
fn snake_path_to_camel_case(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for (i, segment) in path.split('.').enumerate() {
        if i > 0 {
            out.push('.');
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            if i == 0 {
                out.push(first);
            } else {
                out.extend(first.to_uppercase());
            }
            out.push_str(chars.as_str());
        }
    }
    out
}

// Chuyển đổi lỗi DbErr sang AppError
impl From<sea_orm::DbErr> for AppError {
    fn from(inner: sea_orm::DbErr) -> Self {
        AppError::InternalServerError(inner.into())
    }
}
