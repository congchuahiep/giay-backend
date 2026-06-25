use sea_orm::SqlErr;

use crate::core::error::AppError;

pub trait DbErrExt<T> {
    /// Mapping unique constraint violations to [`AppError::BadRequest`]
    ///
    /// # Example
    /// ```
    /// let user = new_user.insert(&state.db).await.check_unique(&[(
    ///     "user_email_key",
    ///     "This email is already registered, please use a different email!",
    /// )])?;
    /// ```
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
