use super::handler;
use crate::core::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(
            handler::list_workspaces,
            handler::create_workspace,
            handler::update_workspace,
            handler::delete_workspace,
        ))
        .routes(routes!(handler::current_workspace))
}
