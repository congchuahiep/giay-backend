use serde::{Deserialize, Deserializer};

use crate::core::error::AppError;

/// Resolves the ID to a UUIDv7, or generates a new one if not provided.
pub fn resolve_v7_id(optional_id: Option<uuid::Uuid>) -> Result<uuid::Uuid, AppError> {
    match optional_id {
        Some(id) if id.get_version() == Some(uuid::Version::SortRand) => Ok(id),
        Some(_) => Err(AppError::BadRequest("ID must be UUIDv7".into())),
        None => Ok(uuid::Uuid::now_v7()),
    }
}

/// Deserializes an optional value, returning `None` if deserialization fails.
///
/// # Example
/// ```json
/// // Client sends:
/// { "icon": null }
/// ```
///
/// ```ignore
/// // Server receives:
/// Icon: None
/// ```
pub fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}
