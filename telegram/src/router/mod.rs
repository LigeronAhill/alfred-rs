use callbacks::callbacks_handler;
use messages::messages_handler;
use shared::models::UserRole;
use teloxide::dispatching::UpdateHandler;
use teloxide::dispatching::dialogue::{self, InMemStorage};
use teloxide::prelude::*;
use teloxide::types::{FileId, InputFile, Message, Update};

mod callbacks;
mod commands;
pub(crate) use commands::commands_handler;
use teloxide::Bot;

use crate::keyboards::{main_menu_inline_keyboard, main_menu_keyboard};
mod messages;

#[derive(Clone, Default)]
pub(crate) enum State {
    #[default]
    Start,
}

pub(crate) type MyDialogue = Dialogue<State, InMemStorage<State>>;

pub(crate) fn router() -> UpdateHandler<anyhow::Error> {
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(messages_handler())
        .branch(callbacks_handler())
}

pub(crate) async fn send_main_menu(
    bot: &Bot,
    msg: &Message,
    user_role: &UserRole,
) -> anyhow::Result<()> {
    let sticker_id = "CAACAgIAAxkBAAEO8sRod-Ra9BIEaWaqgvC3keS8wSQo6AACkwkAAnlc4gneqF_5YHgaODYE";
    let sticker = InputFile::file_id(FileId(sticker_id.to_string()));
    let is_group = msg.chat.is_group() || msg.chat.is_supergroup();
    if is_group {
        let kb = main_menu_inline_keyboard(user_role);
        let mut req = bot.send_sticker(msg.chat.id, sticker).reply_markup(kb);
        req.message_thread_id = msg.thread_id;
        req.await?;
    } else {
        let kb = main_menu_keyboard(user_role);
        let mut req = bot.send_sticker(msg.chat.id, sticker).reply_markup(kb);
        req.message_thread_id = msg.thread_id;
        req.await?;
    }
    Ok(())
}
