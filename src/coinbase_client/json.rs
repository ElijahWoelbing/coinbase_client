use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize};

#[derive(Deserialize)]
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
    pub price: String,
    pub size: String,
    pub order_id: String,
}

#[derive(Deserialize, Debug)]
pub struct OrderBook<T> {
    pub bids: Vec<T>,
    pub asks: Vec<T>,
    pub sequence: u64,
}

#[derive(Deserialize, Debug)]
pub struct Ticker {
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub bid: String,
    pub ask: String,
    pub volume: String,
    #[serde(deserialize_with = "deserialize_datetime")]
    pub time: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
pub struct HistoricRate {
    pub time: u64,
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}
#[derive(Deserialize, Debug)]
pub struct Trade {
    #[serde(deserialize_with = "deserialize_datetime")]
    pub time: DateTime<Utc>,
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub side: String,
}

#[derive(Deserialize, Debug)]
pub struct TwentyFourHourStats {
    pub open: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub last: String,
    pub volume_30day: String,
}

// some field are ompited when a single currency is returned hence the Options enum
#[derive(Deserialize, Debug)]
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
#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
pub struct Time {
    #[serde(deserialize_with = "deserialize_datetime")]
    pub iso: DateTime<Utc>,
    pub epoch: f64,
}

struct DateTimeFromCustomFormatVisitor;

pub fn deserialize_datetime<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(DateTimeFromCustomFormatVisitor)
}

impl<'de> de::Visitor<'de> for DateTimeFromCustomFormatVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a datetime string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.fZ") {
            Ok(ndt) => Ok(DateTime::from_utc(ndt, Utc)),
            Err(e) => Err(E::custom(format!("Parse error {} for {}", e, value))),
        }
    }
}
