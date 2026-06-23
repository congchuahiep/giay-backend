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
use uuid::Uuid;

pub struct ActiveWorkspace {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
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

        println!("workspace_slug: {}", workspace_slug);

        // Cache check (optional, có thể thêm Redis sau)
        // Query workspace + membership trong 1 câu (JOIN)
        let context =
            service::resolve_workspace_context(&state.db, &workspace_slug, &user.id).await?;

        Ok(ActiveWorkspace {
            id: context.id,
            slug: context.slug,
            name: context.name,
            user_role: if let Some(role) = context.user_role {
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
