use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::callbacks::{
    CALCULATE, CALCULATE_CALLBACK, NEWS, NEWS_CALLBACK, PRODUCTS, PRODUCTS_CALLBACK,
};

pub(super) fn main_menu_keyboard() -> KeyboardMarkup {
    let news_button = KeyboardButton::new(NEWS);
    let products_button = KeyboardButton::new(PRODUCTS);
    let calculate_button = KeyboardButton::new(CALCULATE);
    let result = KeyboardMarkup::default()
        .append_row(vec![news_button])
        .append_row(vec![products_button, calculate_button])
        .resize_keyboard()
        .persistent()
        .selective();
    return result;
}
pub(super) fn main_menu_inline_keyboard() -> InlineKeyboardMarkup {
    let news_button = InlineKeyboardButton::callback(NEWS, NEWS_CALLBACK);
    let products_button = InlineKeyboardButton::callback(PRODUCTS, PRODUCTS_CALLBACK);
    let calculate_button = InlineKeyboardButton::callback(CALCULATE, CALCULATE_CALLBACK);
    let result = InlineKeyboardMarkup::default()
        .append_row(vec![news_button])
        .append_row(vec![products_button, calculate_button]);
    return result;
}
