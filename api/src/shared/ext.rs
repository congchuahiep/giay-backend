use sea_orm::SqlErr;
use serde::de::DeserializeOwned;

use crate::core::error::AppError;

/// Instructions for how to lookup an entity by a path param.
///
/// Implement this trait on a MARKER TYPE (ZST), NOT on an Entity.
///
/// # Example
/// ```ignore
/// pub struct ByToken;
///
/// impl ColumnLookup<workspace_invitation::Entity> for ByToken {
///     type Entity = workspace_invitation::Entity;
///     type ValueType = Uuid;
///
///     fn column() -> workspace_invitation::Column {
///         workspace_invitation::Column::Token
///     }
///
///     fn param_name() -> &'static str {
///         "token"
///     }
/// }
/// ```
///
/// Or quickly implement [`ColumnLookup`] by using the [`column_lookup`] macro.
///
/// ```ignore
/// column_lookup!(
///     ByToken => workspace_invitation::Entity {
///         column: workspace_invitation::Column::Token,
///         value_type: Uuid,
///         param: "token",
///     }
/// );
/// ```
pub trait ColumnLookup {
    type Entity: sea_orm::EntityTrait;
    type ValueType: DeserializeOwned + Into<sea_orm::Value> + Send + Sync;
    fn column() -> <Self::Entity as sea_orm::EntityTrait>::Column;
    fn param_name() -> &'static str;
}

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
