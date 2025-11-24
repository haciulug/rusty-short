pub mod api;
pub mod cache;
pub mod config;
pub mod domain;
pub mod observability;
pub mod repository;
pub mod services;

#[cfg(test)]
pub async fn create_test_app() -> axum::Router {
    use std::sync::Arc;
    use sqlx::postgres::PgPoolOptions;

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://rustyshort:rustyshort@localhost:5432/rustyshort".to_string());

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let cache = cache::LinkCache::new(100, 60);
    let repository = repository::LinkRepository::new(db_pool);
    let link_service = Arc::new(services::LinkService::new(
        repository,
        cache,
        "http://localhost:8080".to_string(),
    ));

    let app_state = api::AppState { link_service };
    api::create_router(app_state)
}

