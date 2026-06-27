use super::{workspace, invitation, membership};
use crate::core::state::AppState;
use utoipa_axum::{router::OpenApiRouter, routes};

pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        // Core Workspace
        .routes(routes!(
            workspace::handler::list_workspaces,
            workspace::handler::create_workspace
        ))
        .routes(routes!(
            workspace::handler::get_workspace,
            workspace::handler::update_workspace,
            workspace::handler::delete_workspace,
        ))
        .routes(routes!(workspace::handler::current_workspace))
        // Invitations
        .routes(routes!(
            invitation::handler::send_invitation,
            invitation::handler::list_invitations
        ))
        .routes(routes!(invitation::handler::revoke_invitation))
        .routes(routes!(invitation::handler::resend_invitation))
        .routes(routes!(
            invitation::handler::accept_invitation,
            invitation::handler::preview_invitation
        ))
        
        // Membership
        .routes(routes!(membership::handler::list_members))
        .routes(routes!(
            membership::handler::update_member_role,
            membership::handler::remove_member,
        ))
        .routes(routes!(membership::handler::leave_workspace))
}
