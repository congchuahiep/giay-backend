use super::service;
use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
};
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use entity::sea_orm_active_enums::WorkspaceRole;

use entity::workspace;

pub struct ActiveWorkspace {
    pub workspace: workspace::Model,
    pub user_role: WorkspaceRole,
}

impl FromRequestParts<AppState> for ActiveWorkspace {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(workspace_slug): Path<String> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Missing workspace slug".into()))?;

        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        // Cache check (optional, có thể thêm Redis sau)
        // Query workspace + membership trong 1 câu (JOIN)
        let (workspace, user_role_opt) = service::resolve_workspace_context(
            &state.db,
            &mut state.redis.clone(),
            &workspace_slug,
            &user.id,
        )
        .await?;

        Ok(ActiveWorkspace {
            workspace,
            user_role: if let Some(role) = user_role_opt {
                role
            } else {
                return Err(AppError::Forbidden);
            },
        })
    }
}

pub struct WorkspaceMember(pub ActiveWorkspace);

impl FromRequestParts<AppState> for WorkspaceMember {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ws = ActiveWorkspace::from_request_parts(parts, state).await?;
        Ok(WorkspaceMember(ws))
    }
}

pub struct WorkspaceOwner(pub ActiveWorkspace);

impl FromRequestParts<AppState> for WorkspaceOwner {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ws = ActiveWorkspace::from_request_parts(parts, state).await?;

        match ws.user_role {
            WorkspaceRole::Owner | WorkspaceRole::Moderator => Ok(WorkspaceOwner(ws)),
            _ => Err(AppError::Forbidden),
        }
    }
}
