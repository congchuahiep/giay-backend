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
/// Or quickly implement [`ColumnLookup`] by using the [`define_lookups`] macro.
///
/// ```ignore
/// define_lookups! {
///     WorkspaceBySlug => workspace { Slug: String = "workspace_slug" },
///     InvitationByToken => workspace_invitation { Token: uuid::Uuid = "token" },
///     InvitationById => workspace_invitation { Id: uuid::Uuid = "invitation_id" },
///     UserByEmail => user { Email: String = "email" },
/// }
/// ```
pub trait ColumnLookup {
    type Entity: sea_orm::EntityTrait;
    type ValueType: serde::de::DeserializeOwned + Into<sea_orm::Value> + Send + Sync;
    fn column() -> <Self::Entity as sea_orm::EntityTrait>::Column;
    fn param_name() -> &'static str;
}

/// Define multiple lookups at once with a clean syntax.
///
/// # Example
/// ```
/// define_lookups! {
///     WorkspaceBySlug => workspace { Slug: String = "workspace_slug" },
///     InvitationByToken => workspace_invitation { Token: uuid::Uuid = "token" },
///     InvitationById => workspace_invitation { Id: uuid::Uuid = "invitation_id" },
/// }
/// ```
macro_rules! define_lookups {
    (
        $(
            $struct_name:ident => $module:ident { $column:ident : $val_type:ty = $param_name:literal }
        ),* $(,)?
    ) => {
        $(
            pub struct $struct_name;
            impl $crate::ColumnLookup for $struct_name {
                type Entity = crate::models::$module::Entity;
                type ValueType = $val_type;
                fn column() -> <Self::Entity as sea_orm::EntityTrait>::Column {
                    crate::models::$module::Column::$column
                }
                fn param_name() -> &'static str {
                    $param_name
                }
            }
        )*
    };
}

define_lookups! {
    WorkspaceBySlug => workspace { Slug: String = "workspace_slug" },
    InvitationByToken => workspace_invitation { Token: uuid::Uuid = "token" },
    InvitationById => workspace_invitation { Id: uuid::Uuid = "invitation_id" },
    UserByEmail => user { Email: String = "email" },
}
