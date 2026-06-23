use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: MultiplexedConnection,
    pub jwt_secret: String,
}
