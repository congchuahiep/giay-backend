use super::{
    dto::TokenResponse,
    jwt::{AccessClaims, RefreshClaims, create_token},
};
use crate::core::error::AppError;
use chrono::{Duration, Utc};
use jsonwebtoken::EncodingKey;
use redis::{AsyncTypedCommands, aio::MultiplexedConnection};
use tracing::error;
use uuid::Uuid;

#[inline]
pub fn session_cache_key(jti: &uuid::Uuid) -> String {
    format!("session:{}", jti)
}

/// Phát hành một cặp Access Token và Refresh Token mới cho người dùng.
///
/// Hàm này thực hiện hai nhiệm vụ chính:
/// 1. Tạo JWT Access Token (thời hạn ngắn) dùng để xác thực các request.
/// 2. Tạo Refresh Token (thời hạn dài) và lưu thông tin phiên bản (session) xuống Redis.
///
/// # Errors
///
/// Trả về [`AppError::InternalServerError`] trong các trường hợp sau:
/// * Xảy ra lỗi trong quá trình mã hóa (encode) chuỗi JWT.
/// * Không thể lưu thông tin phiên bản Refresh Token mới vào cơ sở dữ liệu.
pub async fn issue_tokens(
    redis: &mut MultiplexedConnection,
    user: &entity::user::Model,
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

    let cache_key = session_cache_key(&refresh_jti);
    if let Err(e) = redis
        .set_ex(cache_key, user.id.to_string(), 7 * 24 * 60 * 60)
        .await
    {
        error!(
            "Failed to cache refresh token for: {} \n {}",
            refresh_jti, e
        );
        return Err(AppError::InternalServerError(anyhow::anyhow!(
            "Failed to cache refresh token"
        )));
    }

    Ok(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
    })
}
