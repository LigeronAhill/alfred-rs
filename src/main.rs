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
    let pg_storage = PgStorage::init(settings.database_settings).await?;
    pg_storage.close().await;
    Ok(())
}
