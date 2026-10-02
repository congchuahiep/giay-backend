use entity::{sea_orm_active_enums::UserRole, user};
use o2o::o2o;
use sea_orm::FromQueryResult;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, o2o, FromQueryResult)]
#[from_owned(user::Model)]
#[serde(rename_all = "camelCase")]
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
