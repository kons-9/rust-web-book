use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use adapter::database::connect_database_with;
use adapter::redis::RedisClient;
use anyhow::Context;
use anyhow::Result;
use api::route::{auth, v1};
use axum::Router;
use axum::http::Method;
use registry::AppRegistry;
use shared::{config::AppConfig, env::which};
use tokio::net::TcpListener;
use tower_http::LatencyUnit;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    bootstrap().await
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_headers(tower_http::cors::Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(tower_http::cors::Any)
}

fn init_logger() -> Result<()> {
    let log_level = match which() {
        shared::env::Environment::Development => "debug",
        shared::env::Environment::Production => "info",
    };

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| log_level.into());
    let subscriber = tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_target(false);
    tracing_subscriber::registry()
        .with(env_filter)
        .with(subscriber)
        .try_init()?;
    Ok(())
}

async fn bootstrap() -> Result<()> {
    let app_config = AppConfig::new()?;

    let pool = connect_database_with(&app_config.database);

    let kv = Arc::new(RedisClient::new(&app_config.redis)?);

    let registry = AppRegistry::new(pool, kv, app_config);

    let app = Router::new()
        .merge(v1::routes())
        .merge(auth::routes())
        .layer(cors())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .latency_unit(LatencyUnit::Millis),
                ),
        )
        .with_state(registry);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!("Listening on http://{}", addr);

    axum::serve(listener, app)
        .await
        .context("Failed to start the server")
        .inspect_err(|e| {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Unexpected error"
            )
        })
}
