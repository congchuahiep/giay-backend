use crate::core::error::AppError;
use chrono::Utc;
use entity::{sea_orm_active_enums::WorkspaceRole, workspace_invitation};
use o2o::o2o;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
pub struct CreateInvitationRequest {
    #[schema(example = "colleague@example.com")]
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[schema(value_type = String, example = "member")]
    pub role: WorkspaceRole,
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
