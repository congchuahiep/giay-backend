use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Lỗi máy chủ nội bộ")]
    InternalServerError(#[from] anyhow::Error),

    #[error("Không tìm thấy tài nguyên")]
    NotFound,

    #[error("Bạn không có quyền truy cập")]
    Forbidden,

    #[error("Bạn chưa đăng nhập")]
    Unauthorized,

    #[error("Email hoặc mật khẩu không đúng")]
    InvalidCredentials,

    #[error("Phiên đăng nhập đã hết hạn")]
    TokenExpired,

    #[error("{0}")]
    BadRequest(String),
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
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        };

        let body = Json(json!({
            "error": self.to_string()
        }));

        (status, body).into_response()
    }
}

// Chuyển đổi lỗi DbErr sang AppError
impl From<sea_orm::DbErr> for AppError {
    fn from(inner: sea_orm::DbErr) -> Self {
        AppError::InternalServerError(inner.into())
    }
}
