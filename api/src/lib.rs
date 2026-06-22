pub mod auth;
pub mod core;
pub mod router;
pub mod shared;
pub mod user;
pub mod workspace;

use axum::Router;
use std::net::SocketAddr;
use tracing::info;

use crate::core::config::Config;
use crate::core::state::AppState;

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::from_env()?;
    let _ = config.setup_tracing();
    let db = config.connect_db().await?;

    let state = AppState {
        db,
        jwt_secret: config.jwt_secret.clone(),
    };

    let app_router = router::build(state);
    serve(app_router, config.socket_addr()?).await
}

async fn serve(app_router: Router, addr: SocketAddr) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("listening on http://{addr}");
    axum::serve(listener, app_router).await?;
    Ok(())
}
