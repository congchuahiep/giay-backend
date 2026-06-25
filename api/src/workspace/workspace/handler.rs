use super::dto;
use crate::{
    assign_patch,
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
    shared::{DbErrExt, ValidatedJson, resolve_v7_id},
    workspace::{WorkspaceMember, WorkspaceOwner},
};
use axum::{Json, extract::State, http::StatusCode};
use entity::{
    SoftDeleteQueryExt, sea_orm_active_enums::WorkspaceRole, workspace, workspace_membership,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, TransactionTrait,
};

#[utoipa::path(
    get,
    path = "",
    tag = "Workspace",
    summary = "List all workspaces the user is a member of",
    responses(
        (status = 200, description = "List of workspaces", body = dto::WorkspaceResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_workspaces(
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<dto::WorkspaceResponse>>, AppError> {
    let workspaces = workspace::Entity::find()
        .join(
            JoinType::InnerJoin,
            workspace::Relation::WorkspaceMembership.def(),
        )
        .active()
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
    request_body = dto::CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = dto::WorkspaceResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_workspace(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<dto::CreateWorkspaceRequest>,
) -> Result<(StatusCode, Json<dto::WorkspaceResponse>), AppError> {
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
    .check_unique(&[(
        "workspace_slug_key",
        "This slug is already taken, please choose a different one!",
    )])?;

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
    path = "/{workspace_slug}",
    tag = "Workspace",
    summary = "Get a workspace detail by slug",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
    ),
    responses(
        (status = 200, description = "Workspace found", body = dto::WorkspaceResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_workspace(
    WorkspaceMember(ws): WorkspaceMember,
) -> Result<Json<dto::WorkspaceResponse>, AppError> {
    Ok(Json(ws.workspace.into()))
}

#[utoipa::path(
    patch,
    path = "/{workspace_slug}",
    tag = "Workspace",
    summary = "Update workspace",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
    ),
    responses(
        (status = 200, description = "Workspace updated"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_workspace(
    WorkspaceOwner(ws): WorkspaceOwner,
    State(state): State<AppState>,
    Json(payload): Json<dto::UpdateWorkspaceRequest>,
) -> Result<(StatusCode, Json<dto::WorkspaceResponse>), AppError> {
    let mut updated_workspace = workspace::ActiveModel {
        id: Set(ws.workspace.id),
        ..Default::default()
    };

    assign_patch!(updated_workspace, payload, [name, slug, icon]);

    let result = updated_workspace.update(&state.db).await.check_unique(&[(
        "workspace_slug_key",
        "This slug is already taken, please choose a different one!",
    )])?;

    Ok((StatusCode::OK, Json(result.into())))
}

#[utoipa::path(
    delete,
    path = "/{workspace_slug}",
    tag = "Workspace",
    summary = "Delete workspace",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
    ),
    responses(
        (status = 204, description = "Workspace deleted"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_workspace(
    WorkspaceOwner(ws): WorkspaceOwner,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let deleted_workspace = workspace::ActiveModel {
        id: Set(ws.workspace.id),
        deleted_at: Set(Some(chrono::Utc::now().into())),
        ..Default::default()
    };

    deleted_workspace.update(&state.db).await?;

    return Ok(StatusCode::NO_CONTENT);
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
        (status = 200, description = "Current workspace", body = dto::ActiveWorkspaceResponse),
    ),
    security(("bearer_auth" = []))
)]
pub async fn current_workspace(
    WorkspaceMember(ws): WorkspaceMember,
) -> Result<(StatusCode, Json<dto::ActiveWorkspaceResponse>), AppError> {
    Ok((StatusCode::OK, Json(ws.into())))
}
