use entity::{ext::AsStr, user};
use serde::Serialize;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub role: String,
    pub is_active: bool,
}

impl From<user::Model> for UserResponse {
    fn from(model: user::Model) -> Self {
        Self {
            id: model.id,
            email: model.email,
            first_name: model.first_name,
            last_name: model.last_name,
            role: model.role.as_str(),
            is_active: model.is_active,
        }
    }
}
