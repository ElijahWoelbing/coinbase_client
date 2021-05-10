use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Ticker {
    trade_id: u64,
    price: String,
    size: String,
    bid: String,
    ask: String,
    volume: String,
    time: String,
}

#[derive(Deserialize, Debug)]
pub struct HistoricRate {
    time: u64,
    low: f64,
    high: f64,
    open: f64,
    close: f64,
    volume: f64,
}
#[derive(Deserialize, Debug)]
pub struct Trade {
    time: String,
    trade_id: u64,
    price: String,
    size: String,
    side: String,
}

#[derive(Deserialize, Debug)]
pub struct TwentyFourHourStats {
    open: String,
    high: String,
    low: String,
    volume: String,
    last: String,
    volume_30day: String,
}

#[derive(Deserialize, Debug)]
pub struct Currency {
    id: String,
    name: String,
    min_size: String,
    status: String,
    message: String,
    max_precision: String,
    convertible_to: Vec<String>,
    details: CurrencyDetails,
}

#[derive(Deserialize, Debug)]
struct CurrencyDetails {
    r#type: String, // use raw identifier to allow reserved keyword
    symbol: String,
    network_confirmations: u64,
    sort_order: u64,
    crypto_address_link: String,
    crypto_transaction_link: String,
    push_payment_methods: Vec<String>,
    group_types: Vec<String>,
    display_name: String,
    processing_time_seconds: f64,
    min_withdrawal_amount: f64,
    max_withdrawal_amount: f64,
}
