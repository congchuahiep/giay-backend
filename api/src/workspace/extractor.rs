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
    pub workspace_id: Uuid,
    pub workspace_slug: String,
    pub user_role: WorkspaceRole,
}

impl FromRequestParts<AppState> for ActiveWorkspace {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(slug): Path<String> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Thiếu workspace slug".into()))?;

        let user = AuthenticatedUser::from_request_parts(parts, state).await?;

        // Cache check (optional, có thể thêm Redis sau)
        // Query workspace + membership trong 1 câu (JOIN)
        let context = service::resolve_workspace_context(&state.db, &slug, &user.id).await?;

        Ok(ActiveWorkspace {
            workspace_id: context.workspace_id,
            workspace_slug: context.workspace_slug,
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
