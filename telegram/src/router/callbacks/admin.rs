use crate::{
    UsersClient,
    keyboards::{admin_panel_inline_keyboard, admin_panel_keyboard},
};
use anyhow::Result;
use shared::models::UserRole;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
};

pub(super) async fn handler(bot: Bot, q: CallbackQuery) -> Result<()> {
    if let Some(msg) = q.regular_message() {
        bot.delete_message(msg.chat.id, msg.id).await?;
        if msg.chat.is_group() || msg.chat.is_supergroup() {
            let kb = admin_panel_inline_keyboard();
            let mut req = bot
                .send_message(msg.chat.id, "Панель администратора")
                .reply_markup(kb);
            req.message_thread_id = msg.thread_id;
            req.await?;
        } else {
            let kb = admin_panel_keyboard();
            bot.send_message(msg.chat.id, "Панель администратора")
                .reply_markup(kb)
                .await?;
        }
    }
    Ok(())
}

pub(super) async fn list_all_users_handler(
    bot: Bot,
    q: CallbackQuery,
    users_client: UsersClient,
) -> Result<()> {
    if let Some(msg) = q.regular_message() {
        let user = &q.from;
        if users_client
            .get_user(user.id.0)
            .await
            .is_ok_and(|r| r.user_role == UserRole::Admin)
        {
            bot.delete_message(msg.chat.id, msg.id).await?;
            let limit = 10;
            let offset = 0;
            let users = users_client.list_users(limit, offset).await?;
            for user in users {
                let text = format!(
                    "Имя: {name} -> текущая роль: {role}",
                    name = user.user_name,
                    role = user.user_role
                );
                let make_admin_button =
                    InlineKeyboardButton::callback("Назначить администратором", "make_admin");
                let make_employee_button =
                    InlineKeyboardButton::callback("Назначить сотрудником", "make_employee");
                let make_guest_button =
                    InlineKeyboardButton::callback("Назначить гостем", "make_guest");
                let kb = match user.user_role {
                    UserRole::Admin => InlineKeyboardMarkup::new(vec![
                        vec![make_employee_button],
                        vec![make_guest_button],
                    ]),
                    UserRole::Employee => InlineKeyboardMarkup::new(vec![
                        vec![make_admin_button],
                        vec![make_guest_button],
                    ]),
                    UserRole::Guest => InlineKeyboardMarkup::new(vec![
                        vec![make_admin_button],
                        vec![make_employee_button],
                    ]),
                };
                let mut req = bot.send_message(msg.chat.id, text).reply_markup(kb);
                req.message_thread_id = msg.thread_id;
                req.await?;
            }
            // let next_button = InlineKeyboardButton::callback("Следующая страница", "next_users");
        }
    }
    Ok(())
}
