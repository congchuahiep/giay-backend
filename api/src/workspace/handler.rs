use super::{
    WorkspaceMember,
    dto::{ActiveWorkspaceResponse, CreateWorkspaceRequest, WorkspaceResponse},
};
use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
    shared::{ValidatedJson, resolve_v7_id},
};
use axum::{Json, extract::State, http::StatusCode};
use entity::{sea_orm_active_enums::WorkspaceRole, workspace, workspace_membership};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, SqlErr, TransactionTrait,
};

#[utoipa::path(
    get,
    path = "",
    tag = "Workspace",
    summary = "List all workspaces the user is a member of",
    responses(
        (status = 200, description = "List of workspaces", body = WorkspaceResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

#[utoipa::path(
    post,
    path = "",
    tag = "Workspace",
    summary = "Create a new workspace",
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_workspace(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    ValidatedJson(payload): ValidatedJson<CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), AppError> {
    let txn = state.db.begin().await?;

    let new_ws = workspace::ActiveModel {
        id: Set(resolve_v7_id(payload.id)?),
        name: Set(payload.name),
        slug: Set(payload.slug),
        icon: Set(payload.icon),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| {
        if let Some(SqlErr::UniqueConstraintViolation(msg)) = e.sql_err() {
            if msg.contains("workspace_slug_key") {
                return AppError::BadRequest(
                    "Slug này đã tồn tại, vui lòng chọn slug khác!".into(),
                );
            }
        }
        AppError::from(e)
    })?;

    workspace_membership::ActiveModel {
        workspace_id: Set(new_ws.id),
        user_id: Set(user.id),
        role: Set(WorkspaceRole::Owner),
        ..Default::default()
    }
    .insert(&txn)
    .await?;

    txn.commit().await?;
    Ok((StatusCode::CREATED, Json(new_ws.into())))
}

#[utoipa::path(
    get,
    path = "/{workspace_slug}/current",
    tag = "Workspace",
    summary = "Get current workspace",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
    ),
    responses(
        (status = 200, description = "Current workspace", body = ActiveWorkspaceResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn current_workspace(
    WorkspaceMember(ws): WorkspaceMember,
) -> Result<(StatusCode, Json<ActiveWorkspaceResponse>), AppError> {
    Ok((StatusCode::OK, Json(ws.into())))
}
