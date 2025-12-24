use std::net::{Ipv4Addr, SocketAddr};

use axum::{Router, extract::State, http::StatusCode, routing::get};
use sqlx::{PgPool, postgres::PgConnectOptions};
use tokio::net::TcpListener;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn health_check_db(State(db): State<PgPool>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_cfg = DatabaseConfig {
        host: "localhost".to_string(),
        port: 5432,
        username: "app".to_string(),
        password: "passwd".to_string(),
        database: "app".to_string(),
    };
    let connection_pool = connect_database_with(database_cfg);
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/db", get(health_check_db))
        .with_state(connection_pool);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);

    let listener = TcpListener::bind(addr).await?;

    println!("Listening on http://{}", addr);

    Ok(axum::serve(listener, app).await?)
}

struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl From<DatabaseConfig> for PgConnectOptions {
    fn from(config: DatabaseConfig) -> Self {
        PgConnectOptions::new()
            .host(&config.host)
            .port(config.port)
            .username(&config.username)
            .password(&config.password)
            .database(&config.database)
    }
}

fn connect_database_with(cfg: DatabaseConfig) -> PgPool {
    let options = cfg.into();
    PgPool::connect_lazy_with(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn health_check_works() {
        let status_code = health_check().await;
        assert_eq!(status_code, StatusCode::OK);
    }
    #[sqlx::test]
    async fn health_check_db_works(db: PgPool) {
        let status_code = health_check_db(State(db)).await;
        assert_eq!(status_code, StatusCode::OK);
    }
}
