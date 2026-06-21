use crate::core::error::AppError;
use entity::{sea_orm_active_enums::WorkspaceRole, workspace, workspace_membership};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType, QueryFilter,
    QuerySelect, RelationTrait, sea_query::IntoCondition,
};
use uuid::Uuid;

pub struct WorkspaceContext {
    pub workspace_id: Uuid,
    pub workspace_slug: String,
    pub user_role: Option<WorkspaceRole>,
}

#[derive(FromQueryResult)]
struct WorkspaceQueryResult {
    id: Uuid,
    slug: String,
    user_role: Option<WorkspaceRole>,
}

/// Resolves the workspace context for a given slug and user ID.
pub async fn resolve_workspace_context(
    database: &DatabaseConnection,
    slug: &str,
    user_id: &Uuid,
) -> Result<WorkspaceContext, AppError> {
    // Cache check (optional, có thể thêm Redis sau)
    // Query workspace + membership trong 1 câu (LEFT JOIN)
    let user_id_val = *user_id;
    let workspace_data = workspace::Entity::find()
        .filter(workspace::Column::Slug.eq(slug))
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
        .select_only()
        .column(workspace::Column::Id)
        .column(workspace::Column::Slug)
        .column_as(workspace_membership::Column::Role, "user_role")
        .into_model::<WorkspaceQueryResult>()
        .one(database)
        .await
        .map_err(AppError::from)?;

    let workspace_data = workspace_data.ok_or(AppError::NotFound)?;

    Ok(WorkspaceContext {
        workspace_id: workspace_data.id,
        workspace_slug: workspace_data.slug,
        user_role: workspace_data.user_role,
    })
}
