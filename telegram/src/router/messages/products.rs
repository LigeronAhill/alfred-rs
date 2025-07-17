use anyhow::Result;
use teloxide::prelude::*;

use crate::keyboards::{products_inline_keyboard, products_keyboard};

pub(super) async fn handler(bot: Bot, msg: Message) -> Result<()> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    if msg.chat.is_group() || msg.chat.is_supergroup() {
        let kb = products_inline_keyboard();
        let mut req = bot
            .send_message(msg.chat.id, "Панель товаров")
            .reply_markup(kb);
        req.message_thread_id = msg.thread_id;
        req.await?;
    } else {
        let kb = products_keyboard();
        bot.send_message(msg.chat.id, "Панель товаров")
            .reply_markup(kb)
            .await?;
    }
    Ok(())
}
