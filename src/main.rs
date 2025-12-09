use tracing::{debug, info};

#[tokio::main]
async fn main() {
    if std::env::var("ALF_PRODUCTION").is_ok() {
        alfred::logger::init(tracing::Level::INFO);
    } else {
        alfred::logger::init(tracing::Level::DEBUG);
    }
    info!("Hello from Alfred!");
    let settings = alfred::settings::init();
    debug!("Settings:\n{settings:#?}");
}
