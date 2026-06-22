use super::handler;
use crate::core::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handler::login))
        .routes(routes!(handler::register))
        .routes(routes!(handler::logout))
        .routes(routes!(handler::refresh_token))
        .routes(routes!(handler::revoke_token))
}
