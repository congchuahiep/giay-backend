use entity::sea_orm_active_enums::WorkspaceRole;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema, FromQueryResult)]
pub struct MemberResponse {
    pub id: Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    #[schema(value_type = String, example = "member")]
    pub role: WorkspaceRole,
    #[schema(value_type = String)]
    pub joined_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[derive(serde::Deserialize, ToSchema)]
pub struct UpdateMemberRoleRequest {
    #[schema(value_type = String, example = "moderator")]
    pub role: WorkspaceRole,
}
