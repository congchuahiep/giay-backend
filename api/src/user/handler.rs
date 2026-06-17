use super::dto::UserResponse;
use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
};
use axum::{Json, extract::State};
use entity::user;
use sea_orm::EntityTrait;

pub async fn me(
    State(state): State<AppState>,
    auth: AuthenticatedUser,
) -> Result<Json<UserResponse>, AppError> {
    let user = user::Entity::find_by_id(auth.id)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(user.into()))
}
