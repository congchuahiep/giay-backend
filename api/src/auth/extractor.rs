use std::convert::Infallible;

use axum::{
    Json, RequestPartsExt,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    extract::CookieJar,
    headers::{Authorization, authorization::Bearer},
};
use entity::sea_orm_active_enums::UserRole;
use uuid::Uuid;

use super::jwt::{AccessClaims, verify_token};
use crate::core::state::AppState;
use crate::{auth::dto::RefreshRequest, core::error::AppError};

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub id: Uuid,
    pub role: UserRole,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth) = parts.extensions.get::<AuthenticatedUser>().cloned() {
            return Ok(auth);
        }

        // Kiểm tra token trong cookie (trường hợp sử dụng webapp) hoặc header (trường hợp sử dụng
        // API/Tauri desktop app)
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let mut token = jar.get("access_token").map(|c| c.value().to_string());
        if token.is_none() {
            if let Ok(TypedHeader(Authorization(bearer))) =
                parts.extract::<TypedHeader<Authorization<Bearer>>>().await
            {
                token = Some(bearer.token().to_string());
            }
        }

        let token_str = token.ok_or(AppError::Unauthorized)?;

        let claims: AccessClaims = verify_token(&token_str, &state.jwt_secret)?;

        Ok(AuthenticatedUser {
            id: claims.sub,
            role: claims.role,
        })
    }
}

pub struct AdminUser {
    pub id: Uuid,
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthenticatedUser::from_request_parts(parts, state).await?;

        if auth_user.role != UserRole::Admin {
            return Err(AppError::Forbidden);
        }

        Ok(AdminUser { id: auth_user.id })
    }
}

pub struct ExtractedRefreshToken(pub Option<String>);

impl FromRequest<AppState> for ExtractedRefreshToken {
    type Rejection = Infallible;

    async fn from_request(req: Request, state: &AppState) -> Result<Self, Self::Rejection> {
        let (parts, body) = req.into_parts();

        let jar = CookieJar::from_headers(&parts.headers);
        if let Some(cookie) = jar.get("refresh_token") {
            return Ok(ExtractedRefreshToken(Some(cookie.value().to_string())));
        }

        let req = Request::from_parts(parts, body);
        if let Ok(Json(payload)) = Json::<RefreshRequest>::from_request(req, state).await {
            return Ok(ExtractedRefreshToken(Some(payload.refresh_token)));
        }

        Ok(ExtractedRefreshToken(None))
    }
}
