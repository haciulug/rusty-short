mod api;
mod cache;
mod config;
mod domain;
mod observability;
mod repository;
mod services;

use anyhow::Result;
use axum::{http::Method, middleware as axum_middleware, routing::get};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    api::{create_router, AppState},
    cache::LinkCache,
    config::Config,
    observability::{init_logging, setup_metrics_recorder, track_metrics},
    repository::LinkRepository,
    services::LinkService,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let config = Config::from_env()?;
    tracing::info!("Configuration loaded successfully");

    let db_pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&config.database_url)
        .await?;
    tracing::info!("Database connection pool established");

    sqlx::migrate!("./migrations").run(&db_pool).await?;
    tracing::info!("Database migrations completed");

    let cache = LinkCache::new(config.cache_max_capacity, config.cache_ttl);
    tracing::info!(
        "Cache initialized with capacity: {} and TTL: {}s",
        config.cache_max_capacity,
        config.cache_ttl
    );

    let repository = LinkRepository::new(db_pool.clone());
    let link_service = Arc::new(LinkService::new(
        repository.clone(),
        cache,
        config.base_url.clone(),
    ));

    let app_state = AppState {
        link_service,
        repository,
    };

    let metrics_handle = setup_metrics_recorder();
    tracing::info!("Metrics recorder initialized");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = create_router(app_state)
        .route("/metrics", get(move || async move { metrics_handle.render() }))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum_middleware::from_fn(track_metrics))
                .layer(CompressionLayer::new())
                .layer(cors),
        );

    let listener = tokio::net::TcpListener::bind(&config.server_addr).await?;
    tracing::info!("Server listening on {}", config.server_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

