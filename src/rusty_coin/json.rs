use std::u64;

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    pub id: String,
    pub display_name: String,
    pub base_currency: String,
    pub quote_currency: String,
    pub base_increment: String,
    pub quote_increment: String,
    pub base_min_size: String,
    pub base_max_size: String,
    pub min_market_funds: String,
    pub max_market_funds: String,
    pub status: String,
    pub status_message: String,
    pub cancel_only: bool,
    pub limit_only: bool,
    pub post_only: bool,
    pub trading_disabled: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Bid(String, String, i32);
#[derive(Serialize, Deserialize, Debug)]
pub struct Ask(String, String, i32);
#[derive(Serialize, Deserialize, Debug)]
pub struct BookEntry {
    pub price: String,
    pub size: String,
    pub num_orders: u64,
}

#[derive(Deserialize, Debug)]
pub struct FullBookEntry {
    price: String,
    size: String,
    order_id: String,
}

#[derive(Deserialize, Debug)]
pub struct OrderBook<T> {
    bids: Vec<T>,
    asks: Vec<T>,
    sequence: u64,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Ticker {
    trade_id: u64,
    price: String,
    size: String,
    bid: String,
    ask: String,
    volume: String,
    time: String,
}
