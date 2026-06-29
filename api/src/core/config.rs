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
    pub cookie_secure: bool,
    pub log_level: String,

    pub host: String,
    pub port: u16,

    pub cors_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");

        let smtp_host = env::var("SMTP_HOST").expect("SMTP_HOST must be set");
        let smtp_username = env::var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
        let smtp_password = env::var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
        let from_email = env::var("FROM_EMAIL").expect("FROM_EMAIL must be set");

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(!cfg!(debug_assertions));
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_owned());

        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_owned());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let cors_origins_str = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173".to_owned());
        let cors_origins: Vec<String> = cors_origins_str
            .split(',')
            .map(|s| s.trim().to_owned())
            .collect();

        Ok(Self {
            database_url,
            redis_url,

            smtp_host,
            smtp_username,
            smtp_password,
            from_email,

            jwt_secret,
            cookie_secure,
            log_level,

            host,
            port,

            cors_origins,
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
