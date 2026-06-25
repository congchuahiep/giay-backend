use super::{
    WorkspaceMember,
    dto::{ActiveWorkspaceResponse, CreateWorkspaceRequest, WorkspaceResponse},
};
use crate::{
    assign_patch,
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
    shared::{DbErrExt, PathModelLookup, ValidatedJson, resolve_v7_id},
    workspace::{
        WorkspaceModerator, WorkspaceOwner,
        dto::{
            CreateInvitationRequest, InvitationPreviewResponse, InvitationResponse,
            InvitationStatus, UpdateWorkspaceRequest,
        },
        ext, service,
    },
};
use axum::{Json, extract::State, http::StatusCode};
use entity::{
    SoftDeleteQueryExt, sea_orm_active_enums::WorkspaceRole, user, workspace, workspace_membership,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait, SelectExt, TransactionTrait,
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
    user: AuthenticatedUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<WorkspaceResponse>>, AppError> {
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
    request_body = CreateWorkspaceRequest,
    responses(
        (status = 201, description = "Workspace created", body = WorkspaceResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_workspace(
    user: AuthenticatedUser,
    State(state): State<AppState>,
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
        (status = 200, description = "Workspace found", body = WorkspaceResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_workspace(
    WorkspaceMember(ws): WorkspaceMember,
) -> Result<Json<WorkspaceResponse>, AppError> {
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
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_workspace(
    WorkspaceOwner(ws): WorkspaceOwner,
    State(state): State<AppState>,
    Json(payload): Json<UpdateWorkspaceRequest>,
) -> Result<(StatusCode, Json<WorkspaceResponse>), AppError> {
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
    security(
        ("bearer_auth" = [])
    )
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

#[utoipa::path(
    post,
    path = "/{workspace_slug}/invite",
    tag = "Workspace",
    summary = "Send invitation",
    params(
        ("workspace_slug" = String, Path, description = "The slug of the workspace", example = "my-workspace"),
    ),
    request_body = CreateInvitationRequest,
    responses(
        (status = 201, description = "Invitation sent", body = InvitationResponse),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn send_invitation(
    WorkspaceModerator(aw): WorkspaceModerator,
    State(state): State<AppState>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<(StatusCode, Json<InvitationResponse>), AppError> {
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
        mailer
            .send_invitation(
                &payload.email,
                &aw.workspace.name,
                &invitation.token.to_string(),
            )
            .await
    });

    Ok((StatusCode::CREATED, Json(invitation.into())))
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
        (status = 200, description = "Invitation", body = InvitationPreviewResponse),
        (status = 400, description = "Invalid token or invitation already accepted"),
        (status = 404, description = "Invitation not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn preview_invitation(
    PathModelLookup(invitation): PathModelLookup<ext::InvitationByToken>,
    State(state): State<AppState>,
) -> Result<Json<InvitationPreviewResponse>, AppError> {
    let status = InvitationStatus::from_invitation(&invitation);
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

    Ok(Json(InvitationPreviewResponse {
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
        (status = 200, description = "Invitation accepted", body = InvitationResponse),
        (status = 400, description = "Invalid token or invitation already accepted"),
        (status = 404, description = "Invitation not found"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn accept_invitation(
    auth: AuthenticatedUser,
    PathModelLookup(invitation): PathModelLookup<ext::InvitationByToken>,
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<InvitationResponse>), AppError> {
    let invitation = service::accept_invitation(&state.db, invitation, auth.id).await?;
    Ok((StatusCode::OK, Json(invitation.into())))
}
