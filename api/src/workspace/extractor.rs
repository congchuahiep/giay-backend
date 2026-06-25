use super::service;
use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
};
use axum::{
    extract::{FromRequestParts, Path},
    http::request::Parts,
};
use entity::{WorkspaceBound, sea_orm_active_enums::WorkspaceRole};

use entity::workspace;

#[derive(serde::Deserialize)]
struct WorkspaceSlugParam {
    workspace_slug: String,
}

#[derive(Clone)]
pub struct ActiveWorkspace {
    pub auth: AuthenticatedUser,
    pub workspace: workspace::Model,
    pub user_role: WorkspaceRole,
}

impl ActiveWorkspace {
    /// Returns a query builder for entities that implement [`WorkspaceBound`]
    ///
    /// Example:
    /// ```no_run
    /// use entity::workspace_membership;
    ///
    /// let members = aw
    ///     .bound_query::<workspace_membership::Entity>()
    ///     .all(&state.db)
    ///     .await?;
    /// ```
    pub fn bound_query<E: WorkspaceBound>(&self) -> sea_orm::Select<E> {
        E::find_by_workspace(self.workspace.id)
    }
}

impl FromRequestParts<AppState> for ActiveWorkspace {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(aw) = parts.extensions.get::<ActiveWorkspace>().cloned() {
            return Ok(aw);
        }

        let Path(WorkspaceSlugParam { workspace_slug }) = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Missing workspace slug".into()))?;

        let auth = AuthenticatedUser::from_request_parts(parts, state).await?;

        // Cache check (optional, có thể thêm Redis sau)
        // Query workspace + membership trong 1 câu (JOIN)
        let (workspace, user_role_opt) = service::resolve_workspace_context(
            &state.db,
            &mut state.redis.clone(),
            &workspace_slug,
            &auth.id,
        )
        .await?;

        Ok(ActiveWorkspace {
            auth,
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

pub struct WorkspaceModerator(pub ActiveWorkspace);

impl FromRequestParts<AppState> for WorkspaceModerator {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let ws = ActiveWorkspace::from_request_parts(parts, state).await?;

        match ws.user_role {
            WorkspaceRole::Owner | WorkspaceRole::Moderator => Ok(WorkspaceModerator(ws)),
            _ => Err(AppError::Forbidden),
        }
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
            WorkspaceRole::Owner => Ok(WorkspaceOwner(ws)),
            _ => Err(AppError::Forbidden),
        }
    }
}
