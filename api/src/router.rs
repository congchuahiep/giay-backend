use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
};
use serde_json::{Value, json};
use tower_http::{
    LatencyUnit,
    cors::{AllowOrigin, CorsLayer},
    trace::{DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::core::{config::Config, state::AppState};

pub fn build(state: AppState, config: &Config) -> Router {
    let (router, api) = utoipa_axum::router::OpenApiRouter::new()
        .nest("/api/auth", crate::auth::router())
        .nest("/api/user", crate::user::router())
        .nest("/api/workspace", crate::workspace::router())
        .split_for_parts();

    let origins: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true);

    let mut openapi = crate::core::swagger::ApiDoc::openapi();
    openapi.merge(api);

    router
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi))
        .route("/health", axum::routing::get(health))
        .with_state(state)
        .layer(cors)
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
