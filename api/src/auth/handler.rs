use super::{
    dto::{LoginRequest, RefreshRequest, RegisterRequest, TokenResponse},
    extractor::ExtractedRefreshToken,
    jwt, password, service,
};
use crate::{
    auth::{AdminUser, jwt::RefreshClaims},
    core::{error::AppError, state::AppState},
    shared::{DbErrExt, ValidatedJson},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use redis::AsyncTypedCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use uuid::Uuid;

#[utoipa::path(
    post,
    path = "/login",
    tag = "Authentication",
    request_body = LoginRequest,
    summary = "Login",
    responses(
        (status = 200, description = "Đăng nhập thành công", body = TokenResponse),
        (status = 401, description = "Sai email hoặc mật khẩu")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<(StatusCode, CookieJar, Json<TokenResponse>), AppError> {
    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Email.eq(payload.email))
        .one(&state.db)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    password::verify_password(payload.password.clone(), user.password.clone()).await?;

    let token_response =
        service::issue_tokens(&mut state.redis.clone(), &user, &state.jwt_secret).await?;

    let (access_cookie, refresh_cookie) = jwt::build_cookies(
        token_response.access_token.clone(),
        token_response.refresh_token.clone(),
        state.cookie_secure,
    );

    Ok((
        StatusCode::OK,
        jar.add(access_cookie).add(refresh_cookie),
        Json(token_response),
    ))
}

#[utoipa::path(
    post,
    path = "/register",
    tag = "Authentication",
    summary = "Register a new user",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "Registration successful", body = TokenResponse),
        (status = 400, description = "Invalid data or email already exists")
    )
)]
pub async fn register(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RegisterRequest>,
) -> Result<(StatusCode, Json<TokenResponse>), AppError> {
    let hashed_password = password::hash_password(payload.password.clone()).await?;

    let new_user = entity::user::ActiveModel {
        id: Set(Uuid::now_v7()),
        email: Set(payload.email),
        password: Set(hashed_password),
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        ..Default::default()
    };

    let user = new_user.insert(&state.db).await.check_unique(&[(
        "user_email_key",
        "This email is already registered, please use a different email!",
    )])?;

    let token_response =
        service::issue_tokens(&mut state.redis.clone(), &user, &state.jwt_secret).await?;
    Ok((StatusCode::CREATED, Json(token_response)))
}

#[utoipa::path(
    post,
    path = "/refresh-token",
    tag = "Authentication",
    summary = "Refresh Token",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Refresh token successfully", body = TokenResponse),
        (status = 401, description = "Refresh token is invalid or has been revoked")
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    jar: CookieJar,
    ExtractedRefreshToken(token_opt): ExtractedRefreshToken,
) -> Result<(StatusCode, CookieJar, Json<TokenResponse>), AppError> {
    let token = token_opt.ok_or(AppError::Unauthorized)?;
    let claims: jwt::RefreshClaims = jwt::verify_token(&token, &state.jwt_secret)?;

    // Kiểm tra xem phiên đăng nhập có tồn tại không bằng cách xoá phiên đăng nhập cũ trước đó
    // nếu xoá được thì trước đó có phiên đăng nhập hợp lệ thật!
    let cache_key = service::session_cache_key(&claims.jti);
    let mut redis = state.redis.clone();

    match redis.get(&cache_key).await.unwrap_or(None) {
        Some(saved_id) => {
            if saved_id != claims.sub.to_string() {
                return Err(AppError::Unauthorized);
            }
            if let Err(e) = redis.del(&cache_key).await {
                tracing::error!("Failed to delete cache: {}", e);
                return Err(AppError::InternalServerError(e.into()));
            }
        }
        None => {
            return Err(AppError::Unauthorized);
        }
    }

    let user = entity::user::Entity::find_by_id(claims.sub)
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let token_response = service::issue_tokens(&mut redis, &user, &state.jwt_secret).await?;
    let (access_cookie, refresh_cookie) = jwt::build_cookies(
        token_response.access_token.clone(),
        token_response.refresh_token.clone(),
        state.cookie_secure,
    );

    Ok((
        StatusCode::OK,
        jar.add(access_cookie).add(refresh_cookie),
        Json(token_response),
    ))
}

#[utoipa::path(
    post,
    path = "/logout",
    tag = "Authentication",
    summary = "Logout",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Logout successfully")
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    ExtractedRefreshToken(token_opt): ExtractedRefreshToken,
) -> Result<(StatusCode, CookieJar, Json<serde_json::Value>), AppError> {
    if let Some(t) = token_opt
        && let Ok(claims) = jwt::verify_token::<RefreshClaims>(&t, &state.jwt_secret)
    {
        let cache_key = service::session_cache_key(&claims.jti);
        let mut redis = state.redis.clone();

        if let Err(e) = redis.del(&cache_key).await {
            tracing::error!("Failed to delete cache: {}", e);
        }
    }

    let access_removal = Cookie::build(("access_token", "")).path("/").build();
    let refresh_removal = Cookie::build(("refresh_token", ""))
        .path("/api/auth")
        .build();

    let updated_jar = jar.remove(access_removal).remove(refresh_removal);

    Ok((
        StatusCode::OK,
        updated_jar,
        Json(json!({ "message": "Logout successfully" })),
    ))
}

#[utoipa::path(
    post,
    path = "/revoke-token/{session_id}",
    tag = "Authentication",
    summary = "Revoke Token",
    params(
        ("session_id" = Uuid, Path, description = "The ID of the session to revoke")
    ),
    responses(
        (status = 200, description = "Revoke token successfully"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Session not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn revoke_token(
    _: AdminUser,
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
) -> Result<(), AppError> {
    let cache_key = service::session_cache_key(&session_id);
    let mut redis = state.redis.clone();

    match redis.del(&cache_key).await {
        Ok(effected) if effected == 0 => {
            return Err(AppError::NotFound);
        }
        Err(e) => {
            tracing::error!("Failed to delete cache: {}", e);
            return Err(AppError::InternalServerError(e.into()));
        }
        _ => {}
    }

    Ok(())
}
