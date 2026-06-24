use super::extractor;
use crate::shared::deserialize_some;
use entity::{sea_orm_active_enums::WorkspaceRole, workspace, workspace_invitation};
use o2o::o2o;
use regex::Regex;
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
