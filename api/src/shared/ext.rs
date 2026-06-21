use sea_orm::EntityTrait;
use serde::de::DeserializeOwned;

pub trait LookupColumn: EntityTrait {
    type ValueType: DeserializeOwned + Into<sea_orm::Value> + Send + Sync;
    fn lookup_column() -> Self::Column;
}
