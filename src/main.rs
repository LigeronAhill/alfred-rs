use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    info!("Hello from Alfred!");
    let settings = alfred::settings::init();
    info!("Settings:\n{settings:#?}");
}
