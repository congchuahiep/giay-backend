use crate::core::error::AppError;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: Uuid,
    pub role: String,
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
