pub(crate) mod callbacks;
pub(crate) mod keyboards;
mod router;
mod users_client;
use anyhow::Result;
use router::{State, router};
use teloxide::{dispatching::dialogue::InMemStorage, payloads::DeleteWebhookSetters, prelude::*};
use tracing::info;
pub(crate) use users_client::UsersClient;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_file(true)
        .compact()
        .init();
    info!("Starting bot...");
    let bt = std::env::var("BEARER_TOKEN").map(|t| format!("Bearer {t}"))?;
    let users_client = UsersClient::new(&bt)?;
    let bot = Bot::from_env();
    bot.delete_webhook().drop_pending_updates(true).await?;

    Dispatcher::builder(bot, router())
        .dependencies(dptree::deps![users_client, InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
