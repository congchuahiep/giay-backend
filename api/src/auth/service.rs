use super::{
    dto::TokenResponse,
    jwt::{AccessClaims, RefreshClaims, create_token},
};
use crate::core::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::EncodingKey;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

/// Phát hành một cặp Access Token và Refresh Token mới cho người dùng.
///
/// Hàm này thực hiện hai nhiệm vụ chính:
/// 1. Tạo JWT Access Token (thời hạn ngắn) dùng để xác thực các request.
/// 2. Tạo Refresh Token (thời hạn dài) và lưu thông tin phiên bản (session) xuống cơ sở dữ liệu để
/// quản lý.
///
/// # Errors
///
/// Trả về [`AppError::InternalServerError`] trong các trường hợp sau:
/// * Xảy ra lỗi trong quá trình mã hóa (encode) chuỗi JWT.
/// * Không thể lưu thông tin phiên bản Refresh Token mới vào cơ sở dữ liệu.
pub async fn issue_tokens(
    user: &entity::user::Model,
    db: &DatabaseConnection,
    jwt_secret: &str,
) -> Result<TokenResponse, AppError> {
    let now = Utc::now();
    let exp_access = now + Duration::minutes(15);
    let exp_refresh = now + Duration::days(7);

    let refresh_jti = Uuid::new_v4();
    let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());

    let access_token = create_token(
        &AccessClaims {
            sub: user.id,
            role: user.role.clone(),
            iat: now.timestamp() as usize,
            exp: exp_access.timestamp() as usize,
        },
        &encoding_key,
    )?;

    let refresh_token = create_token(
        &RefreshClaims {
            jti: refresh_jti,
            sub: user.id,
            exp: exp_refresh.timestamp() as usize,
        },
        &encoding_key,
    )?;

    entity::user_session::ActiveModel {
        id: Set(refresh_jti),
        user_id: Set(user.id),
        expires_at: Set(exp_refresh.fixed_offset()),
        ..Default::default()
    }
    .insert(db)
    .await
    .map_err(|_| AppError::InternalServerError(anyhow::anyhow!("Lỗi lưu refresh token")))?;

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
    })
}
