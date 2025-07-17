use anyhow::{Error, Result};
use teloxide::{
    dispatching::UpdateHandler,
    payloads::{SendMessageSetters, SendStickerSetters},
    prelude::*,
    types::{FileId, InputFile},
    utils::command::BotCommands,
};

use crate::keyboards::back_to_main_menu_keyboard;

use super::{MyDialogue, send_main_menu};

pub(crate) fn commands_handler() -> UpdateHandler<Error> {
    teloxide::filter_command::<Command, _>()
        .branch(dptree::case![Command::Start].endpoint(start_command_handler))
        .branch(dptree::case![Command::Help].endpoint(help_command_handler))
        .branch(dptree::case![Command::Cancel].endpoint(cancel_command_handler))
}

/// Поддерживаются следующе команды:
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Показать справку.
    Help,
    /// Открыть главное меню.
    Start,
    /// Отмена.
    Cancel,
}

async fn start_command_handler(
    bot: Bot,
    dialogue: MyDialogue,
    msg: Message,
    users_client: crate::UsersClient,
) -> Result<()> {
    let chat_id = dialogue.chat_id();
    bot.delete_message(chat_id, msg.id).await?;
    dialogue.update(super::State::Start).await?;
    if let Some(user) = &msg.from {
        let user_role = match users_client.get_user(user.id.0).await {
            Ok(user) => user.user_role,
            Err(_) => {
                let user_name = user.full_name();
                let user_role = users_client
                    .register_new_user(user.id.0, user_name.clone())
                    .await?;
                user_role
            }
        };
        send_main_menu(&bot, &msg, &user_role).await?;
    }
    Ok(())
}
async fn help_command_handler(bot: Bot, msg: Message) -> Result<()> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    let kb = back_to_main_menu_keyboard();
    let mut req = bot
        .send_message(msg.chat.id, Command::descriptions().to_string())
        .reply_markup(kb);
    req.message_thread_id = msg.thread_id;
    req.await?;
    Ok(())
}
async fn cancel_command_handler(bot: Bot, dialogue: MyDialogue, msg: Message) -> Result<()> {
    bot.delete_message(msg.chat.id, msg.id).await?;
    let sticker = InputFile::file_id(FileId(String::from(
        "CAACAgIAAxkBAAEO8w9oeIvzNGqHiPAXkULZShjyNNGAhwACkgkAAnlc4gkF7ec-DFfgbjYE",
    )));
    let mut req = bot
        .send_sticker(msg.chat.id, sticker)
        .reply_markup(back_to_main_menu_keyboard());
    req.message_thread_id = msg.thread_id;
    req.await?;
    dialogue.exit().await?;

    Ok(())
}
