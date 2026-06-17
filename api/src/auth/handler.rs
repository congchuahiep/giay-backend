use super::{
    AdminUser,
    dto::{LoginRequest, RefreshRequest, RegisterRequest, TokenResponse},
    jwt::{RefreshClaims, verify_token},
    password, service,
};
use crate::core::{error::AppError, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, SqlErr};
use serde_json::json;
use uuid::Uuid;

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Email.eq(payload.email))
        .one(&state.db)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    password::verify_password(payload.password.clone(), user.password.clone()).await?;

    let token_response = service::issue_tokens(&user, &state.db, &state.jwt_secret).await?;
    Ok(Json(token_response))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let hashed_password = password::hash_password(payload.password.clone()).await?;

    let new_user = entity::user::ActiveModel {
        email: Set(payload.email),
        password: Set(hashed_password),
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        ..Default::default()
    };

    let user = match new_user.insert(&state.db).await {
        Ok(user) => user,
        Err(e) if let Some(SqlErr::UniqueConstraintViolation(msg)) = e.sql_err() => {
            if msg.contains("user_email_key") {
                return Err(AppError::BadRequest(
                    "Email này đã được đăng ký, vui lòng dùng email khác!".to_string(),
                ));
            }
            return Err(AppError::BadRequest(msg));
        }
        Err(e) => {
            return Err(AppError::InternalServerError(anyhow::anyhow!(
                "DB Error: {}",
                e
            )));
        }
    };

    let token_response = service::issue_tokens(&user, &state.db, &state.jwt_secret).await?;
    Ok(Json(token_response))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let claims: RefreshClaims = verify_token(&payload.refresh_token, &state.jwt_secret)?;

    // Kiểm tra xem phiên đăng nhập có tồn tại không bằng cách xoá phiên đăng nhập cũ trước đó
    // nếu xoá được thì trước đó có phiên đăng nhập hợp lệ thật!
    if entity::user_session::Entity::delete_by_id(claims.jti)
        .exec(&state.db)
        .await?
        .rows_affected
        == 0
    {
        return Err(AppError::Unauthorized);
    }

    let user = entity::user::Entity::find_by_id(claims.sub)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let token_response = service::issue_tokens(&user, &state.db, &state.jwt_secret).await?;
    Ok(Json(token_response))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let claims: RefreshClaims = verify_token(&payload.refresh_token, &state.jwt_secret)?;

    entity::user_session::Entity::delete_by_id(claims.jti)
        .exec(&state.db)
        .await?;

    Ok(Json(json!({ "message": "Đăng xuất thành công" })))
}

pub async fn revoke_token(
    State(state): State<AppState>,
    _: AdminUser,
    Path(session_id): Path<Uuid>,
) -> Result<Json<()>, AppError> {
    let result = entity::user_session::Entity::delete_by_id(session_id)
        .exec(&state.db)
        .await?;

    if result.rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    Ok(Json(()))
}
