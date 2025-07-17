mod admin;
mod news;
mod products;
use teloxide::{
    Bot,
    dispatching::{UpdateFilterExt, UpdateHandler},
    dptree,
    prelude::Requester,
    types::{Message, Update},
};

use crate::{
    UsersClient,
    callbacks::{ADMIN, BACK_TO_MAIN_MENU, LIST_USERS, NEWS, PRODUCTS},
};

use super::{commands_handler, send_main_menu};

pub fn messages_handler() -> UpdateHandler<anyhow::Error> {
    Update::filter_message()
        .branch(commands_handler())
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some_and(|c| c == BACK_TO_MAIN_MENU))
                .endpoint(back_to_main_menu_handler),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some_and(|c| c == ADMIN))
                .endpoint(admin::handler),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some_and(|c| c == NEWS))
                .endpoint(news::handler),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some_and(|c| c == PRODUCTS))
                .endpoint(products::handler),
        )
        .branch(
            dptree::filter(|msg: Message| msg.text().is_some_and(|c| c == LIST_USERS))
                .endpoint(admin::list_all_users_handler),
        )
}
async fn back_to_main_menu_handler(
    bot: Bot,
    msg: Message,
    users_client: UsersClient,
) -> anyhow::Result<()> {
    if let Some(ref user) = msg.from {
        let user_role = users_client.get_user(user.id.0).await?.user_role;
        bot.delete_message(msg.chat.id, msg.id).await?;
        send_main_menu(&bot, &msg, &user_role).await?;
    }
    Ok(())
}
