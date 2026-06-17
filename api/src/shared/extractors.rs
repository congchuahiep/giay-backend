use axum::extract::{FromRequestParts, Path};
use sea_orm::{EntityTrait, PrimaryKeyTrait};
use serde::de::DeserializeOwned;

use crate::core::{error::AppError, state::AppState};

/// Extractor tự động truy vấn model từ cơ sở dữ liệu dựa trên ID truyền vào từ đường dẫn
/// (Path parameter)
///
/// # Example
///
/// ```
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
