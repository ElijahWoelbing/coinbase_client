use serde;

#[derive(serde::Deserialize, Debug)]
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

#[derive(serde::Deserialize, Debug)]
pub struct BookEntry {
    pub price: String,
    pub size: String,
    pub num_orders: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct FullBookEntry {
    pub price: String,
    pub size: String,
    pub order_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct OrderBook<T> {
    pub bids: Vec<T>,
    pub asks: Vec<T>,
    pub sequence: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Ticker {
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub bid: String,
    pub ask: String,
    pub volume: String,
    pub time: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct HistoricRate {
    pub time: u64,
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}
#[derive(serde::Deserialize, Debug)]
pub struct Trade {
    pub time: String,
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub side: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct TwentyFourHourStats {
    pub open: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub last: String,
    pub volume_30day: String,
}

// some field are ompited when a single currency is returned hence the Options enum
#[derive(serde::Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    pub min_size: String,
    pub status: String,
    pub message: String,
    pub max_precision: String,
    pub convertible_to: Option<Vec<String>>,
    pub details: CurrencyDetails,
}

// some field are ompited when a single currency is returned hence the Options enum
#[derive(serde::Deserialize, Debug)]
pub struct CurrencyDetails {
    pub r#type: String, // use raw identifier to allow reserved keyword
    pub symbol: String,
    pub network_confirmations: u64,
    pub sort_order: u64,
    pub crypto_address_link: String,
    pub crypto_transaction_link: String,
    pub push_payment_methods: Vec<String>,
    pub group_types: Vec<String>,
    pub display_name: Option<String>,
    pub processing_time_seconds: Option<f64>,
    pub min_withdrawal_amount: f64,
    pub max_withdrawal_amount: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Time {
    pub iso: String,
    pub epoch: f64,
}

#[derive(serde::Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub currency: String,
    pub balance: String,
    pub available: String,
    pub hold: String,
    pub profile_id: String,
    pub trading_enabled: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct OrderResponse {
    pub id: String,
    pub price: Option<String>,
    pub size: Option<String>,
    pub product_id: String,
    pub side: String,
    pub stp: String,
    pub r#type: String,
    pub time_in_force: Option<String>,
    pub post_only: bool,
    pub created_at: String,
    pub fill_fees: String,
    pub filled_size: String,
    pub executed_value: String,
    pub status: String,
    pub settled: bool,
}

#[derive(Debug)]
pub enum Order {
    Limit {
        size: f64,
        price: f64,
        side: OrderSide,
        product_id: String,
        time_in_force: TimeInForce,
    },
    Market {
        size_or_funds: SizeOrFunds,
        side: OrderSide,
        product_id: String,
    },
    Stop {
        size: f64,
        price: f64,
        side: OrderSide,
        product_id: String,
        stop: OrderStop,
        stop_price: f64,
    },
}

impl Order {
    pub fn limit(
        size: f64,
        price: f64,
        side: OrderSide,
        product_id: &str,
        time_in_force: TimeInForce,
    ) -> Self {
        Self::Limit {
            size,
            price,
            side,
            product_id: product_id.to_string(),
            time_in_force,
        }
    }

    pub fn market(size_or_funds: SizeOrFunds, side: OrderSide, product_id: &str) -> Self {
        Self::Market {
            size_or_funds,
            side,
            product_id: product_id.to_string(),
        }
    }

    pub fn stop(
        size: f64,
        price: f64,
        side: OrderSide,
        product_id: &str,
        stop: OrderStop,
        stop_price: f64,
    ) -> Self {
        Self::Stop {
            size,
            price,
            side,
            product_id: product_id.to_string(),
            stop,
            stop_price,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}
#[derive(Clone, Copy, Debug)]
pub enum OrderStop {
    Loss,
    Entry,
}

#[derive(Clone, Copy, Debug)]
pub enum SizeOrFunds {
    Size(f64),
    Funds(f64),
}

#[derive(Clone, Copy, Debug)]
pub enum TimeInForce {
    GoodTillCancel {
        post_only: bool,
    },
    GoodTillTime {
        cancel_after: CancelAfter,
        post_only: bool,
    },
    ImmediateOrCancel,
    FillOrKill,
}

#[derive(Clone, Copy, Debug)]
pub enum CancelAfter {
    Minute,
    Hour,
    Day,
}

impl serde::Serialize for OrderStop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Loss => serializer.serialize_str("loss"),
            Self::Entry => serializer.serialize_str("entry"),
        }
    }
}

impl serde::Serialize for TimeInForce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::GoodTillCancel { post_only: _ } => serializer.serialize_str("GTC"),
            Self::GoodTillTime {
                cancel_after: _,
                post_only: _,
            } => serializer.serialize_str("GTT"),
            Self::ImmediateOrCancel => serializer.serialize_str("IOC"),
            Self::FillOrKill => serializer.serialize_str("FOK"),
        }
    }
}

impl serde::Serialize for CancelAfter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Minute => serializer.serialize_str("min"),
            Self::Hour => serializer.serialize_str("hour"),
            Self::Day => serializer.serialize_str("day"),
        }
    }
}

impl serde::Serialize for Order {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            Order::Market {
                size_or_funds,
                side,
                ref product_id,
            } => {
                #[derive(serde::Serialize)]
                struct MarketOrder<'a> {
                    r#type: &'a str,
                    side: OrderSide,
                    product_id: &'a str,
                    size: Option<SizeOrFunds>,
                    funds: Option<SizeOrFunds>,
                }

                let funds = match size_or_funds {
                    SizeOrFunds::Funds(_) => Some(size_or_funds),
                    _ => None,
                };
                let size = match size_or_funds {
                    SizeOrFunds::Size(_) => Some(size_or_funds),
                    _ => None,
                };

                MarketOrder {
                    r#type: "market",
                    side,
                    product_id,
                    size: size,
                    funds: funds,
                }
                .serialize(serializer)
            }
            Order::Limit {
                size,
                price,
                side,
                ref product_id,
                time_in_force,
            } => {
                let cancel_after = match time_in_force {
                    TimeInForce::GoodTillTime {
                        cancel_after,
                        post_only: _,
                    } => Some(cancel_after),
                    _ => None,
                };

                let post_only = match time_in_force {
                    TimeInForce::GoodTillTime {
                        cancel_after: _,
                        post_only,
                    } => Some(post_only),
                    TimeInForce::GoodTillCancel { post_only } => Some(post_only),
                    _ => None,
                };
                #[derive(serde::Serialize)]
                struct OrderLimit<'a> {
                    r#type: &'a str,
                    side: OrderSide,
                    size: f64,
                    price: f64,
                    product_id: &'a str,
                    time_in_force: TimeInForce,
                    cancel_after: Option<CancelAfter>,
                    post_only: Option<bool>,
                }

                OrderLimit {
                    r#type: "limit",
                    side,
                    size,
                    price,
                    product_id,
                    time_in_force: time_in_force,
                    cancel_after,
                    post_only,
                }
                .serialize(serializer)
            }
            Order::Stop {
                size,
                price,
                side,
                ref product_id,
                stop,
                stop_price,
            } => {
                #[derive(serde::Serialize)]
                struct StopOrder<'a> {
                    r#type: &'a str,
                    side: OrderSide,
                    size: f64,
                    price: f64,
                    product_id: &'a str,
                    stop: OrderStop,
                    stop_price: f64,
                }

                StopOrder {
                    r#type: "stop",
                    side,
                    size,
                    price,
                    product_id,
                    stop,
                    stop_price,
                }
                .serialize(serializer)
            }
        }
    }
}

impl serde::Serialize for OrderSide {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            OrderSide::Buy => serializer.serialize_str("buy"),
            OrderSide::Sell => serializer.serialize_str("sell"),
        }
    }
}

impl serde::Serialize for SizeOrFunds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            Self::Size(size) => serializer.serialize_f64(size),
            Self::Funds(funds) => serializer.serialize_f64(funds),
        }
    }
}
