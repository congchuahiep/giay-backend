use axum::{Json, extract::State};
use entity::{workspace, workspace_membership};
use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
};

use super::dto::WorkspaceResponse;

pub async fn list_workspaces(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
    let workspaces = workspace::Entity::find()
        .join(
            JoinType::InnerJoin,
            workspace::Relation::WorkspaceMembership.def(),
        )
        .filter(workspace_membership::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?;

    Ok(Json(workspaces.into_iter().map(Into::into).collect()))
}
