use axum::{Json, Router};
use serde_json::{Value, json};
use tower_http::{
    LatencyUnit,
    cors::CorsLayer,
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::core::state::AppState;

pub fn build(state: AppState) -> Router {
    let (router, api) = utoipa_axum::router::OpenApiRouter::new()
        .nest("/api/auth", crate::auth::router())
        .nest("/api/user", crate::user::router())
        .nest("/api/workspace", crate::workspace::router())
        .split_for_parts();

    let mut openapi = crate::core::swagger::ApiDoc::openapi();
    openapi.merge(api);

    router
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi))
        .route("/health", axum::routing::get(health))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::extract::Request| {
                    tracing::info_span!("request", "{} {}", request.method(), request.uri().path())
                })
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}
