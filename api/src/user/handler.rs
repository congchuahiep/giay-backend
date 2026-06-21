use super::dto::UserResponse;
use crate::{
    auth::AuthenticatedUser,
    core::{error::AppError, state::AppState},
};
use axum::{Json, extract::State};
use entity::user;
use sea_orm::EntityTrait;

#[utoipa::path(
    get,
    path = "/api/user/me",
    tag = "User",
    responses(
        (status = 200, description = "Lấy thông tin người dùng thành công"),
        (status = 404, description = "Không tìm thấy người dùng"),
        (status = 500, description = "Lỗi server nội bộ"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
