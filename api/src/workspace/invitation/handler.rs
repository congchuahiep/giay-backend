use super::{dto, service};
use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
    shared::{PathBoundModel, PathModel},
    workspace::WorkspaceModerator,
};
use axum::{Json, extract::State, http::StatusCode};
use entity::{InvitationById, InvitationByToken, user, workspace};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, SelectExt};

#[utoipa::path(
    get,
    path = "/{workspace_slug}/invitations",
    tag = "Invitation",
    summary = "List workspace invitations",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace"),
    ),
    responses(
        (status = 200, description = "List of invitations", body = [dto::InvitationResponse]),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_invitations(
    WorkspaceModerator(aw): WorkspaceModerator,
    State(state): State<AppState>,
) -> Result<Json<Vec<dto::InvitationResponse>>, AppError> {
    let invitations = service::get_workspace_invitations(&state.db, aw.workspace.id).await?;

    Ok(Json(invitations.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/{workspace_slug}/invitations",
    tag = "Workspace",
    summary = "Send invitation",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
    ),
    request_body = dto::CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitation sent", body = dto::InvitationResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn send_invitation(
    WorkspaceModerator(aw): WorkspaceModerator,
    State(state): State<AppState>,
    Json(payload): Json<dto::CreateInvitationRequest>,
) -> Result<(StatusCode, Json<dto::InvitationResponse>), AppError> {
    let invitation = service::create_invitation(
        &state.db,
        aw.auth.id,
        aw.workspace.id,
        &payload.email,
        payload.role,
    )
    .await?;

    let mailer = state.mailer.clone();
    tokio::spawn(async move {
        let _ = mailer
            .send_invitation(
                &payload.email,
                &aw.workspace.name,
                &invitation.token.to_string(),
            )
            .await;
    });

    Ok((StatusCode::CREATED, Json(invitation.into())))
}

#[utoipa::path(
    delete,
    path = "/{workspace_slug}/invitations/{invitation_id}",
    tag = "Invitation",
    summary = "Revoke invitation",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace"),
        ("invitation_id" = uuid::Uuid, Path, description = "The ID of the invitation to revoke"),
    ),
    responses(
        (status = 204, description = "Invitation successfully revoked"),
        (status = 400, description = "Cannot revoke accepted invitation"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn revoke_invitation(
    _: WorkspaceModerator,
    PathBoundModel(invitation): PathBoundModel<InvitationById>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    service::revoke_invitation(&state.db, invitation).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/{workspace_slug}/invitations/{invitation_id}/resend",
    tag = "Invitation",
    summary = "Resend invitation",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
        ("invitation_id" = uuid::Uuid, Path, description = "The ID of the invitation", example = ""),
    ),
    responses(
        (status = 200, description = "Invitation resent", body = dto::InvitationResponse),
        (status = 400, description = "Invitation already accepted or email already a member"),
        (status = 404, description = "Invitation not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn resend_invitation(
    WorkspaceModerator(aw): WorkspaceModerator,
    PathBoundModel(invitation): PathBoundModel<InvitationById>,
    State(state): State<AppState>,
) -> Result<Json<dto::InvitationResponse>, AppError> {
    let invitation = service::resend_invitation(&state.db, aw.auth.id, invitation, None).await?;

    let mailer = state.mailer.clone();
    let to_email = invitation.email.clone();
    let ws_name = aw.workspace.name.clone();
    let token_str = invitation.token.to_string();

    tokio::spawn(async move {
        if let Err(e) = mailer
            .send_invitation(&to_email, &ws_name, &token_str)
            .await
        {
            tracing::error!("Failed to resend invitation email: {}", e);
        }
    });

    Ok(Json(invitation.into()))
}

#[utoipa::path(
    get,
    path = "/invitations/{token}",
    tag = "Invitation",
    summary = "Preview invitation",
    params(
        ("token" = uuid::Uuid, Path, description = "The invitation token", example = "00000000-0000-0000-0000-000000000000"),
    ),
    responses(
        (status = 200, description = "Invitation", body = dto::InvitationPreviewResponse),
        (status = 400, description = "Invalid token or invitation already accepted"),
        (status = 404, description = "Invitation not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn preview_invitation(
    PathModel(invitation): PathModel<InvitationByToken>,
    State(state): State<AppState>,
) -> Result<Json<dto::InvitationPreviewResponse>, AppError> {
    let status = dto::InvitationStatus::from_invitation(&invitation);
    status.is_valid()?;

    let workspace = workspace::Entity::find()
        .filter(workspace::Column::Id.eq(invitation.workspace_id))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let user_exists = user::Entity::find()
        .filter(user::Column::Email.eq(&invitation.email))
        .exists(&state.db)
        .await?;

    Ok(Json(dto::InvitationPreviewResponse {
        workspace_name: workspace.name,
        workspace_slug: workspace.slug,
        workspace_icon: workspace.icon,
        role: invitation.role,
        email: invitation.email,
        user_exists,
    }))
}

#[utoipa::path(
    post,
    path = "/invitations/{token}",
    tag = "Invitation",
    summary = "Accept invitation",
    params(
        ("token" = uuid::Uuid, Path, description = "The invitation token", example = "00000000-0000-0000-0000-000000000000"),
    ),
    responses(
        (status = 200, description = "Invitation accepted", body = dto::InvitationResponse),
        (status = 400, description = "Invalid token or invitation already accepted"),
        (status = 404, description = "Invitation not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn accept_invitation(
    auth: AuthenticatedUser,
    PathModel(invitation): PathModel<InvitationByToken>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<dto::InvitationResponse>), AppError> {
    let invitation = service::accept_invitation(&state.db, invitation, auth.id).await?;
    Ok((StatusCode::OK, Json(invitation.into())))
}
