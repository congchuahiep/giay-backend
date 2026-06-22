use entity::{sea_orm_active_enums::WorkspaceRole, workspace};
use o2o::o2o;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use utoipa::ToSchema;
use validator::Validate;

use crate::workspace::extractor::ActiveWorkspace;

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
#[from_owned(ActiveWorkspace)]
pub struct ActiveWorkspaceResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub slug: String,
    #[schema(value_type = String, example = "owner")]
    pub user_role: WorkspaceRole,
}
