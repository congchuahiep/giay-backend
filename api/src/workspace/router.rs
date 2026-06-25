use super::handler;
use crate::core::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(handler::list_workspaces, handler::create_workspace,))
        .routes(routes!(
            handler::get_workspace,
            handler::update_workspace,
            handler::delete_workspace,
        ))
        .routes(routes!(handler::current_workspace))
        .routes(routes!(handler::send_invitation, handler::list_invitations,))
        .routes(routes!(handler::revoke_invitation))
        .routes(routes!(handler::resend_invitation))
        .routes(routes!(
            handler::accept_invitation,
            handler::preview_invitation
        ))
}
