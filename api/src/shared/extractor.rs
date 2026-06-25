use std::collections::HashMap;

use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Request, rejection::JsonRejection},
};
use sea_orm::{ColumnTrait, EntityTrait, PrimaryKeyTrait, QueryFilter};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{
    core::{error::AppError, state::AppState},
    shared::ColumnLookup,
};

/// Extractor for auto-querying model from database based on ID from path parameter
///
/// # Example
///
/// ```ignore
/// /// GET /products/{id}
/// pub fn get_product(PathModel(product): PathModel<Product>) -> impl IntoResponse {
///     Ok(product)
/// }
/// ```
///
/// # Error
/// - [`AppError::BadRequest`]: If the ID in the path is invalid or not in the correct format.
/// - [`AppError::NotFound`]: If no record is found in the database corresponding to the ID.
/// - [`AppError::InternalServerError`]: If a database error occurs.
pub struct PathModel<E: EntityTrait>(pub E::Model);

impl<E> FromRequestParts<AppState> for PathModel<E>
where
    E: EntityTrait,
    <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: DeserializeOwned + Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        type IdType<E> = <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType;
        let Path(id): Path<IdType<E>> =
            Path::from_request_parts(parts, state).await.map_err(|_| {
                AppError::BadRequest("Path value is invalid or not in the correct format".into())
            })?;

        match E::find_by_id(id).one(&state.db).await {
            Ok(Some(model)) => Ok(PathModel(model)),
            Ok(None) => Err(AppError::NotFound),
            Err(e) => Err(AppError::InternalServerError(e.into())),
        }
    }
}

/// Extractor for auto-querying model from database based on [`ColumnLookup`] from path parameter
///
/// # Example
/// ```
/// lookup!(
///     WorkspaceBySlug => workspace::Entity {
///         column: workspace::Column::Slug,
///         value_type: String,
///         param: "slug",
///     }
/// )
///
/// /// GET /workspaces/{slug}
/// pub async fn get_workspace(
///     PathModelLookup(ws): PathModelLookup<workspace::Entity>
/// ) -> impl IntoResponse { ... }
/// ```
///
/// # Error
/// - [`AppError::BadRequest`]: If the value in the path is invalid or not in the correct format.
/// - [`AppError::NotFound`]: If no record is found in the database corresponding to the value.
/// - [`AppError::InternalServerError`]: If a database error occurs.
pub struct PathModelLookup<L: ColumnLookup>(
    pub <<L as ColumnLookup>::Entity as sea_orm::EntityTrait>::Model,
);

impl<L: ColumnLookup> FromRequestParts<AppState> for PathModelLookup<L> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(params): Path<HashMap<String, String>> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Invalid path".into()))?;

        let raw_value = params.get(L::param_name()).ok_or_else(|| {
            AppError::BadRequest(format!("Missing path param: '{}'", L::param_name()))
        })?;

        // Parse string → ValueType
        // (Dùng serde_json để deserialize từ string, hỗ trợ Uuid, i32, String, v.v.)
        let value: L::ValueType =
            serde_json::from_value(serde_json::Value::String(raw_value.clone()))
                .map_err(|_| AppError::BadRequest("Invalid path param format".into()))?;

        match L::Entity::find()
            .filter(L::column().eq(value))
            .one(&state.db)
            .await
        {
            Ok(Some(model)) => Ok(PathModelLookup(model)),
            Ok(None) => Err(AppError::NotFound),
            Err(e) => Err(AppError::InternalServerError(e.into())),
        }
    }
}

/// This extractor have the same behavior as [`Json`] but with additional validation using the
/// [`validator`] crate.
///
/// Use this instead of [`Json<T>`] when `T` implements [`Validate`].
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize, Validate)]
/// struct ProductRequest {
///     #[validate(length(min = 1))]
///     name: String,
///     price: f64,
/// }
///
/// /// POST /products
/// pub fn create_product(
///     ValidatedJson(product): ValidatedJson<ProductRequest>
/// ) -> impl IntoResponse {
///     Ok(product)
/// }
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        // TODO: Sửa cái này để nó trả lỗi cho mỗi loại trường
        if let Err(errors) = value.validate() {
            let error_message = errors
                .field_errors()
                .into_iter()
                .next()
                .map(|(_, errs)| errs[0].message.as_deref().unwrap_or("Invalid data"))
                .unwrap_or("Invalid data");

            return Err(AppError::BadRequest(error_message.to_string()));
        }

        Ok(ValidatedJson(value))
    }
}
