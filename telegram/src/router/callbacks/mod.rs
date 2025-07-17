mod admin;
mod news;
mod products;
use anyhow::Error;
use teloxide::{
    Bot,
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::Requester,
    types::{CallbackQuery, Update},
};

use crate::{
    UsersClient,
    callbacks::{
        ADMIN_CALLBACK, BACK_TO_MAIN_MENU_CALLBACK, LIST_USERS_CALLBACK, NEWS_CALLBACK,
        PRODUCTS_CALLBACK,
    },
};

use super::send_main_menu;

pub(crate) fn callbacks_handler() -> UpdateHandler<Error> {
    Update::filter_callback_query()
        .branch(
            dptree::filter(|q: CallbackQuery| {
                q.data.as_deref() == Some(BACK_TO_MAIN_MENU_CALLBACK)
            })
            .endpoint(back_to_main_menu_handler),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some(ADMIN_CALLBACK))
                .endpoint(admin::handler),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some(NEWS_CALLBACK))
                .endpoint(news::handler),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some(PRODUCTS_CALLBACK))
                .endpoint(products::handler),
        )
        .branch(
            dptree::filter(|q: CallbackQuery| q.data.as_deref() == Some(LIST_USERS_CALLBACK))
                .endpoint(admin::list_all_users_handler),
        )
}

async fn back_to_main_menu_handler(
    bot: Bot,
    q: CallbackQuery,
    users_client: UsersClient,
) -> anyhow::Result<()> {
    if let Some(message) = q.regular_message() {
        bot.answer_callback_query(q.id.clone()).await?;
        let user_role = users_client.get_user(q.from.id.0).await?.user_role;
        bot.delete_message(message.chat.id, message.id).await?;
        send_main_menu(&bot, message, &user_role).await?;
    }
    Ok(())
}
