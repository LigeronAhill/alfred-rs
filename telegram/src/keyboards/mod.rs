use shared::models::UserRole;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::callbacks::{
    BACK_TO_MAIN_MENU, BACK_TO_MAIN_MENU_CALLBACK, CURRENCIES, CURRENCIES_CALLBACK, LIST_USERS,
    LIST_USERS_CALLBACK, PRODUCTS_PRICES, PRODUCTS_PRICES_CALLBACK, PRODUCTS_STOCK,
    PRODUCTS_STOCK_CALLBACK, WEATHER, WEATHER_CALLBACK,
};

mod admin;
mod employee;
mod guest;

pub(crate) fn main_menu_inline_keyboard(role: &UserRole) -> InlineKeyboardMarkup {
    match role {
        UserRole::Admin => admin::main_menu_inline_keyboard(),
        UserRole::Employee => employee::main_menu_inline_keyboard(),
        UserRole::Guest => guest::main_menu_inline_keyboard(),
    }
}
pub(crate) fn main_menu_keyboard(role: &UserRole) -> KeyboardMarkup {
    match role {
        UserRole::Admin => admin::main_menu_keyboard(),
        UserRole::Employee => employee::main_menu_keyboard(),
        UserRole::Guest => guest::main_menu_keyboard(),
    }
}
pub(crate) fn back_to_main_menu_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup {
        inline_keyboard: vec![vec![InlineKeyboardButton::callback(
            BACK_TO_MAIN_MENU,
            BACK_TO_MAIN_MENU_CALLBACK,
        )]],
    }
}

pub(crate) fn admin_panel_inline_keyboard() -> InlineKeyboardMarkup {
    let users_button = InlineKeyboardButton::callback(LIST_USERS, LIST_USERS_CALLBACK);
    let first_row = vec![users_button];

    let back_button = InlineKeyboardButton::callback(BACK_TO_MAIN_MENU, BACK_TO_MAIN_MENU_CALLBACK);
    let last_row = vec![back_button];
    InlineKeyboardMarkup::default()
        .append_row(first_row)
        .append_row(last_row)
}
pub(crate) fn news_inline_keyboard() -> InlineKeyboardMarkup {
    let currencies_button = InlineKeyboardButton::callback(CURRENCIES, CURRENCIES_CALLBACK);
    let weather_button = InlineKeyboardButton::callback(WEATHER, WEATHER_CALLBACK);
    let first_row = vec![currencies_button, weather_button];
    let back_button = InlineKeyboardButton::callback(BACK_TO_MAIN_MENU, BACK_TO_MAIN_MENU_CALLBACK);
    let last_row = vec![back_button];
    InlineKeyboardMarkup::default()
        .append_row(first_row)
        .append_row(last_row)
}
pub(crate) fn products_inline_keyboard() -> InlineKeyboardMarkup {
    let prices_button = InlineKeyboardButton::callback(PRODUCTS_PRICES, PRODUCTS_PRICES_CALLBACK);
    let stock_button = InlineKeyboardButton::callback(PRODUCTS_STOCK, PRODUCTS_STOCK_CALLBACK);
    let first_row = vec![prices_button, stock_button];
    let back_button = InlineKeyboardButton::callback(BACK_TO_MAIN_MENU, BACK_TO_MAIN_MENU_CALLBACK);
    let last_row = vec![back_button];
    InlineKeyboardMarkup::default()
        .append_row(first_row)
        .append_row(last_row)
}

pub(crate) fn admin_panel_keyboard() -> KeyboardMarkup {
    let users_button = KeyboardButton::new(LIST_USERS);
    let first_row = vec![users_button];
    let back_button = KeyboardButton::new(BACK_TO_MAIN_MENU);
    let last_row = vec![back_button];
    KeyboardMarkup::default()
        .resize_keyboard()
        .one_time_keyboard()
        .append_row(first_row)
        .append_row(last_row)
}
pub(crate) fn news_keyboard() -> KeyboardMarkup {
    let currencies_button = KeyboardButton::new(CURRENCIES);
    let weather_button = KeyboardButton::new(WEATHER);
    let first_row = vec![currencies_button, weather_button];
    let back_button = KeyboardButton::new(BACK_TO_MAIN_MENU);
    let last_row = vec![back_button];
    KeyboardMarkup::default()
        .resize_keyboard()
        .append_row(first_row)
        .append_row(last_row)
}
pub(crate) fn products_keyboard() -> KeyboardMarkup {
    let prices_button = KeyboardButton::new(PRODUCTS_PRICES);
    let stock_button = KeyboardButton::new(PRODUCTS_STOCK);
    let first_row = vec![prices_button, stock_button];
    let back_button = KeyboardButton::new(BACK_TO_MAIN_MENU);
    let last_row = vec![back_button];
    KeyboardMarkup::default()
        .resize_keyboard()
        .append_row(first_row)
        .append_row(last_row)
}
