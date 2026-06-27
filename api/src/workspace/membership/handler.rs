use super::{dto, service};
use crate::{
    core::{error::AppError, state::AppState},
    shared::PathBoundModel,
    workspace::{WorkspaceMember, WorkspaceModerator},
};
use axum::{Json, extract::State, http::StatusCode};
use entity::{MembershipByUserId, WorkspaceBound, user, workspace_membership};
use sea_orm::{JoinType, QuerySelect, RelationTrait};

#[utoipa::path(
    get,
    path = "/{workspace_slug}/members",
    tag = "Membership",
    summary = "List workspace members",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace"),
    ),
    responses(
        (status = 200, description = "List of members", body = [dto::MemberResponse]),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_members(
    WorkspaceMember(aw): WorkspaceMember,
    State(state): State<AppState>,
) -> Result<Json<Vec<dto::MemberResponse>>, AppError> {
    let members = workspace_membership::Entity::find_by_workspace(aw.workspace.id)
        .join(
            JoinType::InnerJoin,
            workspace_membership::Relation::User.def(),
        )
        .column(user::Column::Id)
        .column(user::Column::Email)
        .column(user::Column::FirstName)
        .column(user::Column::LastName)
        .into_model::<dto::MemberResponse>()
        .all(&state.db)
        .await?;

    Ok(Json(members))
}

#[utoipa::path(
    patch,
    path = "/{workspace_slug}/members/{user_id}",
    tag = "Membership",
    summary = "Update member role",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace"),
        ("user_id" = Uuid, Path, description = "The ID of the user"),
    ),
    request_body = dto::UpdateMemberRoleRequest,
    responses(
        (status = 200, description = "Role updated successfully"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_member_role(
    WorkspaceModerator(aw): WorkspaceModerator,
    State(state): State<AppState>,
    PathBoundModel(target_membership): PathBoundModel<MembershipByUserId>,
    Json(payload): Json<dto::UpdateMemberRoleRequest>,
) -> Result<StatusCode, AppError> {
    service::update_member_role(
        &state.db,
        target_membership,
        aw.auth.id,
        aw.user_role,
        payload.role,
    )
    .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/{workspace_slug}/members/{user_id}",
    tag = "Membership",
    summary = "Remove a member from the workspace",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace"),
        ("user_id" = Uuid, Path, description = "The ID of the user to remove"),
    ),
    responses(
        (status = 204, description = "Member removed successfully"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn remove_member(
    WorkspaceModerator(aw): WorkspaceModerator,
    State(state): State<AppState>,
    PathBoundModel(target_membership): PathBoundModel<MembershipByUserId>,
) -> Result<StatusCode, AppError> {
    service::remove_member(&state.db, target_membership, aw.auth.id, aw.user_role).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/{workspace_slug}/members/me",
    tag = "Membership",
    summary = "Leave the workspace",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace"),
    ),
    responses(
        (status = 204, description = "Left the workspace successfully"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn leave_workspace(
    WorkspaceMember(aw): WorkspaceMember,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    service::leave_workspace(&state.db, aw.workspace.id, aw.auth.id, aw.user_role).await?;
    Ok(StatusCode::NO_CONTENT)
}
