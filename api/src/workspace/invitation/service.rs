use super::dto::InvitationStatus;
use crate::{core::error::AppError, shared::DbErrExt};
use chrono::{Duration, Utc};
use entity::{
    WorkspaceBound, sea_orm_active_enums::WorkspaceRole, user, workspace_invitation,
    workspace_membership,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, JoinType, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
    TransactionTrait,
};
use uuid::Uuid;

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
        return resend_invitation(db, invited_by, invitation, Some(role)).await;
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

pub async fn resend_invitation(
    db: &DatabaseConnection,
    invited_by: Uuid,
    invitation: workspace_invitation::Model,
    new_role: Option<WorkspaceRole>,
) -> Result<workspace_invitation::Model, AppError> {
    let status = InvitationStatus::from_invitation(&invitation);

    match status {
        InvitationStatus::Accepted => Err(AppError::BadRequest(
            "Cannot resend an invitation that has already been accepted.".into(),
        )),
        _ => Ok(()),
    }?;

    let mut invitation = invitation.into_active_model();

    if let Some(role) = new_role {
        invitation.role = Set(role);
    }

    invitation.token = Set(Uuid::new_v4());
    invitation.invited_by = Set(Some(invited_by));
    invitation.revoked_at = Set(None);
    invitation.expires_at = Set((Utc::now() + Duration::days(7)).into());

    let inserted = invitation.update(db).await?;

    Ok(inserted)
}

pub async fn accept_invitation(
    db: &DatabaseConnection,
    invitation: workspace_invitation::Model,
    user_id: Uuid,
) -> Result<workspace_invitation::Model, AppError> {
    let txn = db.begin().await?;

    let status = InvitationStatus::from_invitation(&invitation);
    status.is_valid()?;

    let user = user::Entity::find_by_id(user_id)
        .one(&txn)
        .await?
        .map_or(Err(AppError::NotFound), |user| Ok(user))?;

    if user.email.to_lowercase() != invitation.email.to_lowercase() {
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

pub async fn revoke_invitation(
    db: &DatabaseConnection,
    invitation: workspace_invitation::Model,
) -> Result<(), AppError> {
    let status = InvitationStatus::from_invitation(&invitation);

    match status {
        InvitationStatus::Accepted => {
            return Err(AppError::BadRequest(
                "Cannot revoke an invitation that has already been accepted.".into(),
            ));
        }
        InvitationStatus::Revoked => return Ok(()),
        _ => {}
    }

    let mut invitation = invitation.into_active_model();
    invitation.revoked_at = Set(Some(Utc::now().into()));

    invitation.update(db).await?;
    Ok(())
}

pub async fn get_workspace_invitations(
    db: &DatabaseConnection,
    workspace_id: Uuid,
) -> Result<Vec<workspace_invitation::Model>, AppError> {
    let invitations = workspace_invitation::Entity::find_by_workspace(workspace_id)
        .order_by_desc(workspace_invitation::Column::CreatedAt)
        .all(db)
        .await?;

    Ok(invitations)
}
