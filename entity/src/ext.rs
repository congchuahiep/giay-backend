use crate::sea_orm_active_enums::UserRole;

pub trait AsStr {
    fn as_str(&self) -> String;
}

impl AsStr for UserRole {
    fn as_str(&self) -> String {
        match self {
            UserRole::Admin => "admin".to_string(),
            UserRole::User => "user".to_string(),
        }
    }
}
