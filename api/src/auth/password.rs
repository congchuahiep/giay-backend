use crate::core::error::AppError;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                tracing::error!("Failed to hash password: {}", e);
                AppError::InternalServerError(anyhow::anyhow!("Password hash error"))
            })?
            .to_string();

        Ok(password_hash)
    })
    .await
    .map_err(|e| {
        tracing::error!("Blocking task failed: {}", e);
        AppError::InternalServerError(anyhow::anyhow!("Internal process error"))
    })?
}

pub async fn verify_password(
    input_password: String,
    password_hash: String,
) -> Result<(), AppError> {
    tokio::task::spawn_blocking(move || {
        let parsed_hash = PasswordHash::new(&password_hash).map_err(|e| {
            tracing::error!("Failed to parse password hash: {}", e);
            AppError::InternalServerError(anyhow::anyhow!("Invalid hash format"))
        })?;

        if Argon2::default()
            .verify_password(input_password.as_bytes(), &parsed_hash)
            .is_err()
        {
            return Err(AppError::InvalidCredentials);
        }

        Ok(())
    })
    .await
    .map_err(|e| {
        tracing::error!("Blocking task failed: {}", e);
        AppError::InternalServerError(anyhow::anyhow!("Internal process error"))
    })?
}
