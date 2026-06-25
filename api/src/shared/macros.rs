#[macro_export]
macro_rules! assign_patch {
    ($model:expr, $payload:expr, [ $( $field:ident ),+ $(,)? ]) => {
        $(
            if let Some(val) = $payload.$field {
                $model.$field = sea_orm::ActiveValue::Set(val);
            }
        )+
    };
}

/// Generate a marker struct and impl [`super::ColumnLookup`] in one step.
///
/// # Example
/// ```ignore
/// lookup! {
///     ByToken => workspace_invitation::Entity {
///         column: workspace_invitation::Column::Token,
///         value_type: Uuid,
///         param: "token",
///     }
/// }
/// ```
#[macro_export]
macro_rules! lookup {
    ($name:ident => $entity:ty {
        column: $column:expr,
        value_type: $value_type:ty,
        param: $param:literal $(,)?
    }) => {
        pub struct $name;
        impl $crate::shared::ColumnLookup for $name {
            type Entity = $entity;
            type ValueType = $value_type;
            fn column() -> <$entity as sea_orm::EntityTrait>::Column {
                $column
            }
            fn param_name() -> &'static str {
                $param
            }
        }
    };
}
