use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[schema(example = "admin@workspace.com")]
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[schema(example = "Secret123!")]
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[schema(example = "admin@workspace.com")]
    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[schema(example = "Secret123!")]
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,

    #[schema(example = "Steve")]
    #[validate(length(min = 1, message = "First name must be at least 1 character long"))]
    pub first_name: String,

    #[schema(example = "Job")]
    #[validate(length(min = 1, message = "Last name must be at least 1 character long"))]
    pub last_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    #[schema(example = "Bearer")]
    pub token_type: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
