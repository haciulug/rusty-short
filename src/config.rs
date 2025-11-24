use std::net::SocketAddr;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: SocketAddr,
    pub base_url: String,
    pub cache_ttl: u64,
    pub cache_max_capacity: u64,
    pub rate_limit_per_second: u64,
    pub rate_limit_burst_size: u32,
    pub default_redirect_type: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .context("Invalid SERVER_PORT")?;

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            server_addr: format!("{}:{}", host, port)
                .parse()
                .context("Invalid server address")?,
            base_url: std::env::var("BASE_URL")
                .unwrap_or_else(|_| format!("http://{}:{}", host, port)),
            cache_ttl: std::env::var("CACHE_TTL")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .context("Invalid CACHE_TTL")?,
            cache_max_capacity: std::env::var("CACHE_MAX_CAPACITY")
                .unwrap_or_else(|_| "10000".to_string())
                .parse()
                .context("Invalid CACHE_MAX_CAPACITY")?,
            rate_limit_per_second: std::env::var("RATE_LIMIT_PER_SECOND")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid RATE_LIMIT_PER_SECOND")?,
            rate_limit_burst_size: std::env::var("RATE_LIMIT_BURST_SIZE")
                .unwrap_or_else(|_| "50".to_string())
                .parse()
                .context("Invalid RATE_LIMIT_BURST_SIZE")?,
            default_redirect_type: std::env::var("DEFAULT_REDIRECT_TYPE")
                .unwrap_or_else(|_| "301".to_string())
                .parse()
                .context("Invalid DEFAULT_REDIRECT_TYPE")?,
        })
    }
}

