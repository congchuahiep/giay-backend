use entity::{sea_orm_active_enums::UserRole, user};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum UserRoleDto {
    Admin,
    User,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    #[schema(example = "admin@workspace.com")]
    pub email: String,
    #[schema(example = "Steve")]
    pub first_name: String,
    #[schema(example = "Job")]
    pub last_name: String,
    pub role: UserRoleDto,
    #[schema(example = "true")]
    pub is_active: bool,
}

impl From<user::Model> for UserResponse {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            first_name: model.first_name,
            last_name: model.last_name,
            role: match model.role {
                UserRole::Admin => UserRoleDto::Admin,
                UserRole::User => UserRoleDto::User,
            },
            is_active: model.is_active,
        }
    }
}
