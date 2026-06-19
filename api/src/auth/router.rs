use super::handler;
use crate::core::state::AppState;
use axum::{Router, routing::post};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(handler::login))
        .route("/register", post(handler::register))
        .route("/refresh-token", post(handler::refresh_token))
        .route("/revoke-token/{session_id}", post(handler::revoke_token))
}
