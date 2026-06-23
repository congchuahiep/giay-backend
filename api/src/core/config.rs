use std::{env, net::SocketAddr};
use tracing_subscriber::{EnvFilter, fmt};

pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned()),
            port: env::var("PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(8000),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_owned()),
        })
    }

    pub fn setup_tracing(&self) {
        fmt()
            .with_env_filter(EnvFilter::new(&self.log_level))
            .init();
    }

    /// Kết nối database từ URL trong config
    pub async fn connect_db(&self) -> anyhow::Result<sea_orm::DatabaseConnection> {
        sea_orm::Database::connect(&self.database_url)
            .await
            .map_err(Into::into)
    }

    pub async fn connect_redis(&self) -> anyhow::Result<redis::aio::MultiplexedConnection> {
        let client = redis::Client::open(self.redis_url.as_str())?;
        Ok(client.get_multiplexed_async_connection().await?)
    }

    /// Parse host:port thành SocketAddr
    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(Into::into)
    }
}
