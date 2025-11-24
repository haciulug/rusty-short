use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "rustyshort=debug,tower_http=debug,axum=debug,sqlx=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

