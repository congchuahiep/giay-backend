use std::{env, net::SocketAddr};
use tracing_subscriber::{EnvFilter, fmt};

pub struct Config {
    pub database_url: String,
    pub redis_url: String,

    pub smtp_host: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,

    pub jwt_secret: String,
    pub log_level: String,

    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),

            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.gmail.com".to_owned()),
            smtp_username: env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set"),
            smtp_password: env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set"),
            from_email: env::var("FROM_EMAIL")
                .unwrap_or_else(|_| "Giấy <noreply@giay.com>".to_owned()),

            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_owned()),
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned()),
            port: env::var("PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .unwrap_or(8000),
        })
    }

    pub fn setup_tracing(&self) {
        fmt()
            .with_env_filter(EnvFilter::new(&self.log_level))
            .init();
    }

    /// Parse host:port thành SocketAddr
    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(Into::into)
    }
}
