use crate::core::error::AppError;
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

use entity::sea_orm_active_enums::UserRole;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub role: UserRole,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub exp: usize,
}

pub fn create_token(
    claims: &impl Serialize,
    encoding_key: &EncodingKey,
) -> Result<String, AppError> {
    encode(&Header::default(), claims, encoding_key).map_err(|e| {
        tracing::error!("Failed to create token: {}", e);
        AppError::InternalServerError(anyhow::anyhow!("Token creation error"))
    })
}

/// Xác thực token và trả về claims (Generic)
pub fn verify_token<T: DeserializeOwned>(token: &str, secret: &str) -> Result<T, AppError> {
    let token_data = decode::<T>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
        _ => AppError::Unauthorized,
    })?;

    Ok(token_data.claims)
}

pub fn build_cookies(
    access_token: String,
    refresh_token: String,
    is_secure: bool,
) -> (Cookie<'static>, Cookie<'static>) {
    let access_cookie = Cookie::build(("access_token", access_token))
        .path("/")
        .http_only(true)
        .secure(is_secure)
        .same_site(SameSite::Lax)
        .build();

    let refresh_cookie = Cookie::build(("refresh_token", refresh_token))
        // Giới hạn Cookie này chỉ chạy tới endpoint refresh
        .path("/api/auth")
        .http_only(true)
        .secure(is_secure)
        .same_site(SameSite::Strict)
        .build();

    (access_cookie, refresh_cookie)
}
