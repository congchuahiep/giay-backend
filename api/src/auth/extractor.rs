use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use entity::sea_orm_active_enums::UserRole;
use uuid::Uuid;

use super::jwt::{AccessClaims, verify_token};
use crate::core::error::AppError;
use crate::core::state::AppState;

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

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::Unauthorized)?;

        let claims: AccessClaims = verify_token(bearer.token(), &state.jwt_secret)?;

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
