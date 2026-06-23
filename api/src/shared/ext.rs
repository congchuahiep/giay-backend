use sea_orm::{EntityTrait, SqlErr};
use serde::de::DeserializeOwned;

use crate::core::error::AppError;

pub trait LookupColumn: EntityTrait {
    type ValueType: DeserializeOwned + Into<sea_orm::Value> + Send + Sync;
    fn lookup_column() -> Self::Column;
}

pub trait DbErrExt<T> {
    /// Mapping unique constraint violations to [`AppError::BadRequest`]
    fn check_unique(self, constraints: &[(&str, &str)]) -> Result<T, AppError>;
}

impl<T> DbErrExt<T> for Result<T, sea_orm::error::DbErr> {
    fn check_unique(self, constraints: &[(&str, &str)]) -> Result<T, AppError> {
        self.map_err(|e| {
            if let Some(SqlErr::UniqueConstraintViolation(msg)) = e.sql_err() {
                for (key, err_msg) in constraints {
                    if msg.contains(key) {
                        return AppError::BadRequest((*err_msg).into());
                    }
                }
            }
            AppError::from(e)
        })
    }
}
