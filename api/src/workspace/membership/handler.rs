use super::dto::MemberResponse;
use crate::{
    core::{error::AppError, state::AppState},
    workspace::WorkspaceMember,
};
use axum::{Json, extract::State};
use entity::{WorkspaceBound, user, workspace_membership};
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
        (status = 200, description = "List of members", body = [MemberResponse]),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_members(
    WorkspaceMember(aw): WorkspaceMember,
    State(state): State<AppState>,
) -> Result<Json<Vec<MemberResponse>>, AppError> {
    let members = workspace_membership::Entity::find_by_workspace(aw.workspace.id)
        .join(
            JoinType::InnerJoin,
            workspace_membership::Relation::User.def(),
        )
        .column(user::Column::Id)
        .column(user::Column::Email)
        .column(user::Column::FirstName)
        .column(user::Column::LastName)
        .into_model::<MemberResponse>()
        .all(&state.db)
        .await?;

    Ok(Json(members))
}
