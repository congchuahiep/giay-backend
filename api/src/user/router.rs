use super::handler;
use crate::core::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new().route("/me", get(handler::me))
}
