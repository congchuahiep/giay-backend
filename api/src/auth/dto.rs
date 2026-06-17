use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}
