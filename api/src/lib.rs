pub mod auth;
pub mod core;
pub mod shared;
pub mod user;
pub mod workspace;

use axum::Json;
use sea_orm::Database;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::core::config::Config;
use crate::core::state::AppState;

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;

    let db = Database::connect(&config.database_url).await?;

    let state = AppState {
        db,
        jwt_secret: config.jwt_secret,
    };

    let (router, api) = utoipa_axum::router::OpenApiRouter::new()
        .nest("/api/auth", auth::router())
        .nest("/api/user", user::router())
        .split_for_parts();

    let mut openapi = crate::core::swagger::ApiDoc::openapi();
    openapi.merge(api);

    let app = router
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", openapi))
        .route("/health", axum::routing::get(health))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("listening on http://{addr}");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}
