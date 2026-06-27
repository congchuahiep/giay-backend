use crate::core::error::AppError;
use entity::{sea_orm_active_enums::WorkspaceRole, workspace_membership};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel,
    ModelTrait, TransactionTrait,
};
use uuid::Uuid;

pub async fn update_member_role(
    db: &DatabaseConnection,
    target_membership: workspace_membership::Model,
    caller_user_id: Uuid,
    caller_role: WorkspaceRole,
    new_role: WorkspaceRole,
) -> Result<(), AppError> {
    if caller_user_id == target_membership.user_id {
        return Err(AppError::BadRequest(
            "You cannot change your own role".into(),
        ));
    }

    let _can_update = match caller_role {
        WorkspaceRole::Owner => Ok(()),
        WorkspaceRole::Moderator
            if new_role != WorkspaceRole::Owner
                && !matches!(
                    target_membership.role,
                    WorkspaceRole::Owner | WorkspaceRole::Moderator
                ) =>
        {
            Ok(())
        }
        _ => Err(AppError::Forbidden),
    }?;

    match new_role {
        WorkspaceRole::Owner => {
            let caller_membership = workspace_membership::Entity::find_by_id((
                target_membership.workspace_id,
                caller_user_id,
            ))
            .one(db)
            .await?
            .ok_or(AppError::NotFound)?;

            let txn = db.begin().await?;

            let mut target_am = target_membership.into_active_model();
            target_am.role = Set(WorkspaceRole::Owner);
            target_am.update(&txn).await?;

            let mut caller_am = caller_membership.into_active_model();
            caller_am.role = Set(WorkspaceRole::Moderator);
            caller_am.update(&txn).await?;

            txn.commit().await?;
        }
        _ => {
            let mut target_am = target_membership.into_active_model();
            target_am.role = Set(new_role);
            target_am.update(db).await?;
        }
    };

    Ok(())
}

pub async fn remove_member(
    db: &DatabaseConnection,
    target_membership: workspace_membership::Model,
    caller_user_id: Uuid,
    caller_role: WorkspaceRole,
) -> Result<(), AppError> {
    if caller_user_id == target_membership.user_id {
        return Err(AppError::BadRequest(
            "Use the \"Leave Workspace\" instead of removing yourself".into(),
        ));
    }

    match target_membership.role {
        WorkspaceRole::Owner => Err(AppError::Forbidden),
        WorkspaceRole::Moderator if caller_role == WorkspaceRole::Moderator => {
            Err(AppError::Forbidden)
        }
        _ => Ok(()),
    }?;

    target_membership.delete(db).await?;
    Ok(())
}

pub async fn leave_workspace(
    db: &DatabaseConnection,
    workspace_id: Uuid,
    caller_user_id: Uuid,
    caller_role: WorkspaceRole,
) -> Result<(), AppError> {
    if caller_role == WorkspaceRole::Owner {
        return Err(AppError::BadRequest(
            "You cannot leave a workspace as the owner. Please transfer ownership to another user first."
                .into(),
        ));
    }

    let response = workspace_membership::Entity::delete_by_id((workspace_id, caller_user_id))
        .exec(db)
        .await?;

    if response.rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    Ok(())
}
