use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, KeyboardButton, KeyboardMarkup};

use crate::callbacks::{
    CALCULATE, CALCULATE_CALLBACK, LIST_POSTS, LIST_POSTS_CALLBACK, NEW_POST, NEW_POST_CALLBACK,
    NEWS, NEWS_CALLBACK, PRODUCTS, PRODUCTS_CALLBACK,
};

pub(super) fn main_menu_keyboard() -> KeyboardMarkup {
    let news_button = KeyboardButton::new(NEWS);
    let new_post_button = KeyboardButton::new(NEW_POST);
    let posts_button = KeyboardButton::new(LIST_POSTS);
    let stock_button = KeyboardButton::new(PRODUCTS);
    let prices_button = KeyboardButton::new(CALCULATE);
    let result = KeyboardMarkup::default()
        .append_row(vec![news_button])
        .append_row(vec![new_post_button, posts_button])
        .append_row(vec![stock_button, prices_button])
        .resize_keyboard()
        .persistent()
        .selective();
    return result;
}
pub(super) fn main_menu_inline_keyboard() -> InlineKeyboardMarkup {
    let news_button = InlineKeyboardButton::callback(NEWS, NEWS_CALLBACK);
    let new_post_button = InlineKeyboardButton::callback(NEW_POST, NEW_POST_CALLBACK);
    let posts_button = InlineKeyboardButton::callback(LIST_POSTS, LIST_POSTS_CALLBACK);
    let stock_button = InlineKeyboardButton::callback(PRODUCTS, PRODUCTS_CALLBACK);
    let prices_button = InlineKeyboardButton::callback(CALCULATE, CALCULATE_CALLBACK);
    let result = InlineKeyboardMarkup::default()
        .append_row(vec![news_button])
        .append_row(vec![new_post_button, posts_button])
        .append_row(vec![stock_button, prices_button]);
    return result;
}
