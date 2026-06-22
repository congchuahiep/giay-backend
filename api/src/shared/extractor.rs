use axum::{
    Json,
    extract::{FromRequest, FromRequestParts, Path, Request, rejection::JsonRejection},
};
use sea_orm::{ColumnTrait, EntityTrait, PrimaryKeyTrait, QueryFilter};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::{
    core::{error::AppError, state::AppState},
    shared::LookupColumn,
};

/// Extractor tự động truy vấn model từ cơ sở dữ liệu dựa trên ID truyền vào từ đường dẫn
/// (Path parameter)
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
/// - [`AppError::BadRequest`]: Nếu ID trên đường dẫn không hợp lệ hoặc không đúng định dạng.
/// - [`AppError::NotFound`]: Nếu không tìm thấy bản ghi nào trong cơ sở dữ liệu ứng với ID đó.
/// - [`AppError::InternalServerError`]: Nếu xảy ra lỗi truy vấn cơ sở dữ liệu.
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
        let Path(id): Path<IdType<E>> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("ID không hợp lệ".into()))?;

        match E::find_by_id(id).one(&state.db).await {
            Ok(Some(model)) => Ok(PathModel(model)),
            Ok(None) => Err(AppError::NotFound),
            Err(e) => Err(AppError::InternalServerError(e.into())),
        }
    }
}

/// Extractor tự động truy vấn model từ cơ sở dữ liệu dựa trên [`LookupColumn`] được truyền vào
/// từ đường dẫn (Path parameter)
///
/// # Example
/// ```
/// /// GET /workspaces/{slug}
///
/// impl LookupColumn for workspace::Entity {
///     type ValueType = String;
///     fn lookup_column() -> Self::Column {
///         workspace::Column::Slug
///     }
/// }
///
/// pub async fn get_workspace(
///     PathModelBy(ws): PathModelBy<workspace::Entity>
/// ) -> impl IntoResponse { ... }
/// ```
///
/// # Error
/// - [`AppError::BadRequest`]: Nếu ID trên đường dẫn không hợp lệ hoặc không đúng định dạng.
/// - [`AppError::NotFound`]: Nếu không tìm thấy bản ghi nào trong cơ sở dữ liệu ứng với ID đó.
/// - [`AppError::InternalServerError`]: Nếu xảy ra lỗi truy vấn cơ sở dữ liệu.
pub struct PathModelBy<E: LookupColumn>(pub E::Model);

impl<E: LookupColumn> FromRequestParts<AppState> for PathModelBy<E> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Path(value): Path<E::ValueType> = Path::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::BadRequest("Giá trị trên đường dẫn không hợp lệ".into()))?;

        match E::find()
            .filter(E::lookup_column().eq(value))
            .one(&state.db)
            .await
        {
            Ok(Some(model)) => Ok(PathModelBy(model)),
            Ok(None) => Err(AppError::NotFound),
            Err(e) => Err(AppError::InternalServerError(e.into())),
        }
    }
}

/// This extractor have the same behavior as [`Json`] but with additional validation using the
/// [`validator`] crate.
///
/// # Example
///
///
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
