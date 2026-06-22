use entity::workspace;
use o2o::o2o;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateWorkspaceRequest {
    pub id: Option<uuid::Uuid>,
    #[schema(example = "My Team Workspace")]
    pub name: String,
    #[schema(example = "my-workspace")]
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
