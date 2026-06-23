use crate::core::error::AppError;
use entity::{
    SoftDeleteQueryExt, sea_orm_active_enums::WorkspaceRole, workspace, workspace_membership,
};
use redis::{AsyncTypedCommands, aio::MultiplexedConnection};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, FromQueryResult, JoinType, QueryFilter,
    QuerySelect, RelationTrait, sea_query::IntoCondition,
};
use tracing::error;
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
///
/// This function first checks the Redis cache for a precomputed result, and if not found, queries
/// the database to resolve the workspace context. (bypass setting cache if there is an error)
pub async fn resolve_workspace_context(
    database: &DatabaseConnection,
    redis: &mut MultiplexedConnection,
    slug: &str,
    user_id: &Uuid,
) -> Result<(workspace::Model, Option<WorkspaceRole>), AppError> {
    let cache_key = format!("ws:{}:member:{}", slug, user_id);

    let cached: Option<String> = redis.get(&cache_key).await.unwrap_or(None);

    // CACHE HIT
    if let Some(json_str) = cached {
        if let Ok(ctx) =
            serde_json::from_str::<(workspace::Model, Option<WorkspaceRole>)>(&json_str)
        {
            return Ok(ctx);
        }
    }

    // CACHE MISS
    // Query workspace + membership trong 1 câu (LEFT JOIN)
    let user_id_val = *user_id;
    let data = workspace::Entity::find()
        .filter(workspace::Column::Slug.eq(slug))
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
        .column_as(workspace_membership::Column::Role, "user_role")
        .into_model::<WorkspaceQueryResult>()
        .one(database)
        .await
        .map_err(AppError::from)?
        .ok_or(AppError::NotFound)?;

    let result = (
        workspace::Model {
            id: data.id,
            name: data.name,
            slug: data.slug,
            icon: data.icon,
            owner_id: data.owner_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
            deleted_at: data.deleted_at,
        },
        data.user_role,
    );

    if let Ok(json) = serde_json::to_string(&result) {
        let _ = redis.set_ex(&cache_key, json, 300).await.inspect_err(|e| {
            error!(
                "Failed to cache workspace context for: {} \n {}",
                cache_key, e
            )
        });
    }

    Ok(result)
}
