use crate::{core::error::AppError, shared::DbErrExt};
use chrono::{Duration, Utc};
use entity::{
    SoftDeleteQueryExt, WorkspaceBound, sea_orm_active_enums::WorkspaceRole, user, workspace,
    workspace_invitation, workspace_membership,
};
use redis::{AsyncTypedCommands, aio::MultiplexedConnection};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    FromQueryResult, IntoActiveModel, JoinType, ModelTrait, QueryFilter, QuerySelect,
    RelationTrait, TransactionTrait, sea_query::IntoCondition,
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

pub async fn create_invitation(
    db: &DatabaseConnection,
    invited_by: Uuid,
    workspace_id: Uuid,
    email: &str,
    role: WorkspaceRole,
) -> Result<workspace_invitation::Model, AppError> {
    workspace_membership::Entity::find_by_workspace(workspace_id)
        .join(
            JoinType::InnerJoin,
            workspace_membership::Relation::User.def(),
        )
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await?
        .map_or(Ok(()), |_| {
            Err(AppError::BadRequest(
                "This email is already a member of the workspace".into(),
            ))
        })?;

    let existing_invitation = workspace_invitation::Entity::find_by_workspace(workspace_id)
        .filter(workspace_invitation::Column::Email.eq(email))
        .one(db)
        .await?;

    if let Some(invitation) = existing_invitation {
        // Nếu DateTime trong DB lưu dưới dạng DateTimeWithTimeZone, SeaORM thường map ra
        // `chrono::DateTime<FixedOffset>`
        // Chúng ta lấy thời gian hiện tại để so sánh
        let now: DateTimeWithTimeZone = Utc::now().into();

        let is_active = invitation.accepted_at.is_none()
            && invitation.revoked_at.is_none()
            && invitation.expires_at > now;

        if is_active {
            // Đang có 1 lời mời hợp lệ chờ phản hồi
            return Err(AppError::BadRequest(
                "Đã có lời mời đang chờ xử lý cho email này.".into(),
            ));
        } else {
            // Lời mời cũ đã hết hạn, bị thu hồi, hoặc người này đã từng accept rồi bị kick ra.
            // Xóa lời mời cũ để nhường chỗ cho lời mời mới (giải quyết Unique Constraint).
            invitation.delete(db).await?;
        }
    }

    let new_invitation = workspace_invitation::ActiveModel {
        id: Set(Uuid::now_v7()),
        workspace_id: Set(workspace_id),
        email: Set(email.into()),
        role: Set(role),
        token: Set(Uuid::new_v4()),
        invited_by: Set(Some(invited_by)),
        expires_at: Set((Utc::now() + Duration::days(7)).into()),
        ..Default::default()
    };

    let inserted = new_invitation.insert(db).await?;

    Ok(inserted)
}

pub async fn accept_invitation(
    db: &DatabaseConnection,
    invitation: workspace_invitation::Model,
    user_id: Uuid,
) -> Result<workspace_invitation::Model, AppError> {
    let txn = db.begin().await?;

    if invitation.accepted_at.is_some() {
        return Err(AppError::BadRequest(
            "The invitation has already been accepted.".into(),
        ));
    }

    if invitation.expires_at < Utc::now() {
        return Err(AppError::BadRequest(
            "The invitation has been expired.".into(),
        ));
    }

    if invitation.revoked_at.is_some() {
        return Err(AppError::BadRequest(
            "The invitation has been revoked.".into(),
        ));
    }

    let user = user::Entity::find_by_id(user_id)
        .one(&txn)
        .await?
        .map_or(Err(AppError::NotFound), |user| Ok(user))?;
    if user.email != invitation.email {
        return Err(AppError::BadRequest(
            "The invitation is not for this user.".into(),
        ));
    }

    workspace_membership::ActiveModel {
        workspace_id: Set(invitation.workspace_id),
        user_id: Set(user_id),
        role: Set(invitation.role.clone()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .check_unique(&[(
        "workspace_membership_pkey",
        "You are already a member of this workspace.",
    )])?;

    let mut invitation = invitation.into_active_model();
    invitation.accepted_at = Set(Some(Utc::now().into()));
    let invitation = invitation.update(&txn).await?;

    txn.commit().await?;

    return Ok(invitation.into());
}
