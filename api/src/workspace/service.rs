use crate::core::error::AppError;
use entity::{
    SoftDeleteQueryExt, sea_orm_active_enums::WorkspaceRole, workspace, workspace_membership,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType, QueryFilter,
    QuerySelect, RelationTrait, sea_query::IntoCondition,
};
use uuid::Uuid;

use sea_orm::prelude::DateTimeWithTimeZone;

#[derive(FromQueryResult)]
pub struct WorkspaceQueryResult {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub icon: Option<String>,
    pub owner_id: Option<Uuid>,
    pub created_at: Option<DateTimeWithTimeZone>,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub user_role: Option<WorkspaceRole>,
}

/// Resolves the workspace context for a given slug and user ID.
pub async fn resolve_workspace_context(
    database: &DatabaseConnection,
    slug: &str,
    user_id: &Uuid,
) -> Result<(workspace::Model, Option<WorkspaceRole>), AppError> {
    // Cache check (optional, có thể thêm Redis sau)
    // Query workspace + membership trong 1 câu (LEFT JOIN)
    let user_id_val = *user_id;
    let workspace_data = workspace::Entity::find()
        .filter(workspace::Column::Slug.eq(slug))
        // CHÚ Ý: Bổ sung .active() để áp dụng cơ chế xoá mềm
        .active()
        .join(
            JoinType::LeftJoin,
            workspace::Relation::WorkspaceMembership
                .def()
                .on_condition(move |_left, _right| {
                    workspace_membership::Column::UserId
                        .eq(user_id_val)
                        .into_condition()
                }),
        )
        // Không dùng select_only() nữa để lấy TOÀN BỘ cột mặc định của bảng workspace
        .column_as(workspace_membership::Column::Role, "user_role")
        .into_model::<WorkspaceQueryResult>()
        .one(database)
        .await
        .map_err(AppError::from)?;

    let data = workspace_data.ok_or(AppError::NotFound)?;

    let model = workspace::Model {
        id: data.id,
        name: data.name,
        slug: data.slug,
        icon: data.icon,
        owner_id: data.owner_id,
        created_at: data.created_at,
        updated_at: data.updated_at,
        deleted_at: data.deleted_at,
    };

    Ok((model, data.user_role))
}
