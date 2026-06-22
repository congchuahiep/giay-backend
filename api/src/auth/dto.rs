use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
pub struct LoginRequest {
    #[schema(example = "admin@workspace.com")]
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[schema(example = "Secret123!")]
    pub password: String,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct RegisterRequest {
    #[schema(example = "admin@workspace.com")]
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[schema(example = "Secret123!")]
    pub password: String,
    #[schema(example = "Steve")]
    pub first_name: String,
    #[schema(example = "Job")]
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
