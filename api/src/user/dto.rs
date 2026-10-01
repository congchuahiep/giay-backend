use entity::sea_orm_active_enums::UserRole;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, FromQueryResult)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    #[schema(example = "admin@workspace.com")]
    pub email: String,
    #[schema(example = "Steve")]
    pub first_name: String,
    #[schema(example = "Job")]
    pub last_name: String,
    pub role: UserRole,
    #[schema(example = "true")]
    pub is_active: bool,
}
