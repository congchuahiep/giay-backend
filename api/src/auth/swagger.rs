use utoipa::OpenApi;
use super::{dto, handler};

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::login,
        handler::register,
        handler::refresh_token,
        handler::logout,
        handler::revoke_token,
    ),
    components(
        schemas(
            dto::LoginRequest,
            dto::RegisterRequest,
            dto::RefreshRequest,
            dto::TokenResponse,
        )
    )
)]
pub struct AuthApiDoc;
