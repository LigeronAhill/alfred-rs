use std::sync::Arc;
use std::time::Duration;

use alfred::AppResult;
use alfred::storage::PgStorage;

#[tokio::main]
async fn main() -> AppResult<()> {
    if std::env::var("ALF_PRODUCTION").is_ok() {
        alfred::logger::init(tracing::Level::INFO);
    } else {
        alfred::logger::init(tracing::Level::DEBUG);
    }
    tracing::info!("Hello from Alfred!");
    let settings = alfred::settings::init("settings.toml");
    let db_url = settings.database_settings.db_url();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(settings.database_settings.max_connections.unwrap_or(8))
        .idle_timeout(Duration::from_secs(
            settings.database_settings.idle_timeout.unwrap_or(30),
        ))
        .connect(db_url.as_ref())
        .await?;
    let pg_storage = Arc::new(PgStorage::init(pool).await?);
    let users_service = Arc::new(alfred::services::UsersService::new(pg_storage.clone()));
    let state = Arc::new(alfred::AppState::new(users_service));
    let server = alfred::Server::new(settings.server_settings, state);
    if server.start().await.is_err() {
        pg_storage.close().await;
    }
    Ok(())
}
