use mail::Mailer;
use redis::aio::MultiplexedConnection;
use sea_orm::DatabaseConnection;

use crate::core::config;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: MultiplexedConnection,
    pub jwt_secret: String,
    pub mailer: Mailer,
}

impl AppState {
    pub async fn new(config: &config::Config) -> anyhow::Result<Self> {
        let db = sea_orm::Database::connect(&config.database_url).await?;

        let redis = redis::Client::open(config.redis_url.as_str())?
            .get_multiplexed_async_connection()
            .await?;

        let mailer = mail::Mailer::new(
            &config.smtp_host,
            &config.smtp_username,
            &config.smtp_password,
            &config.from_email,
        );

        Ok(Self {
            db,
            redis,
            mailer,
            jwt_secret: config.jwt_secret.clone(),
        })
    }
}
