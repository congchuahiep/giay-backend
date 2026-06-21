use super::{dto, handler};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(handler::me,), components(schemas(dto::UserResponse)))]
pub struct UserApiDoc;
