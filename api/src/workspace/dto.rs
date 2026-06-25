use super::extractor;
use crate::{core::error::AppError, shared::deserialize_some};
use chrono::Utc;
use entity::{sea_orm_active_enums::WorkspaceRole, workspace, workspace_invitation};
use o2o::o2o;
use regex::Regex;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::ToSchema;
use validator::Validate;

static SLUG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-z0-9-]+$").unwrap());

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateWorkspaceRequest {
    pub id: Option<uuid::Uuid>,

    #[schema(example = "My Team Workspace")]
    #[validate(length(min = 1, message = "Cannot be empty"))]
    pub name: String,

    #[schema(example = "my-workspace")]
    #[validate(regex(
        path = *SLUG_REGEX,
        message = "Slug only allows lowercase letters, numbers, and hyphens"
    ))]
    pub slug: String,

    #[schema(example = "🚀")]
    pub icon: Option<String>,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct UpdateWorkspaceRequest {
    #[schema(example = "My Team Workspace")]
    #[validate(length(min = 1, message = "Cannot be empty"))]
    pub name: Option<String>,

    #[schema(example = "my-workspace")]
    #[validate(regex(
        path = *SLUG_REGEX,
        message = "Slug only allows lowercase letters, numbers, and hyphens"
    ))]
    pub slug: Option<String>,

    #[schema(example = "🚀")]
    #[serde(default, deserialize_with = "deserialize_some")]
    pub icon: Option<Option<String>>,
}

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateInvitationRequest {
    #[schema(example = "colleague@example.com")]
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[schema(value_type = String, example = "member")]
    pub role: WorkspaceRole,
}

#[derive(Serialize, ToSchema, o2o)]
#[from_owned(workspace::Model)]
pub struct WorkspaceResponse {
    pub id: uuid::Uuid,
    #[schema(example = "My Workspace")]
    pub name: String,
    #[schema(example = "my-workspace")]
    pub slug: String,
    #[schema(example = "🚀")]
    pub icon: Option<String>,
    pub owner_id: Option<uuid::Uuid>,
    #[from(~.map(|t| t.to_string()))]
    pub created_at: Option<String>,
    #[from(~.map(|t| t.to_string()))]
    pub updated_at: Option<String>,
}

#[derive(Serialize, ToSchema, o2o)]
#[from_owned(extractor::ActiveWorkspace)]
pub struct ActiveWorkspaceResponse {
    #[from(~.into())]
    pub workspace: WorkspaceResponse,

    #[schema(value_type = String, example = "owner")]
    pub user_role: WorkspaceRole,
}

#[derive(Serialize, ToSchema, o2o)]
#[from_owned(workspace_invitation::Model)]
pub struct InvitationResponse {
    pub id: uuid::Uuid,
    pub workspace_id: uuid::Uuid,
    pub email: String,

    #[schema(value_type = String)]
    pub role: WorkspaceRole,

    pub token: uuid::Uuid,

    #[from(~.map(|t| t.to_string()))]
    pub created_at: Option<String>,

    #[from(~.to_string())]
    pub expires_at: String,

    #[from(~.map(|t| t.to_string()))]
    pub accepted_at: Option<String>,

    #[from(~.map(|t| t.to_string()))]
    pub revoked_at: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct InvitationPreviewResponse {
    #[schema(example = "My Workspace")]
    pub workspace_name: String,
    #[schema(example = "my-workspace")]
    pub workspace_slug: String,
    #[schema(example = "🚀")]
    pub workspace_icon: Option<String>,

    #[schema(value_type = String, example = "member")]
    pub role: WorkspaceRole,

    pub email: String,

    pub user_exists: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

impl InvitationStatus {
    /// Converts a workspace invitation model to an invitation status.
    pub fn from_invitation(inv: &workspace_invitation::Model) -> Self {
        if inv.accepted_at.is_some() {
            return Self::Accepted;
        }
        if inv.revoked_at.is_some() {
            return Self::Revoked;
        }
        let now: DateTimeWithTimeZone = Utc::now().into();
        if inv.expires_at < now {
            return Self::Expired;
        }
        Self::Pending
    }

    /// Returns an error if the invitation is not valid (expired, revoked, or already accepted).
    pub fn is_valid(&self) -> Result<(), AppError> {
        match self {
            InvitationStatus::Expired => Err(AppError::BadRequest("Invitation has expired".into())),
            InvitationStatus::Revoked => {
                Err(AppError::BadRequest("Invitation has been revoked".into()))
            }
            InvitationStatus::Accepted => Err(AppError::BadRequest(
                "Invitation has already been accepted".into(),
            )),
            _ => Ok(()),
        }
    }
}
