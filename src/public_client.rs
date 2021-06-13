use super::{
    deserialize_response, deserialize_to_date, deserialize_to_f64, COINBASE_API_URL,
    COINBASE_SANDBOX_API_URL,
};
use crate::error::Error;
use chrono::{DateTime, Utc};
use reqwest;
use serde;

/// `PublicClient provides public market data
pub struct PublicClient {
    reqwest_client: reqwest::Client,
    url: &'static str,
}

impl PublicClient {
    pub fn new() -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            url: COINBASE_API_URL,
        }
    }

    pub fn new_sandbox() -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            url: COINBASE_SANDBOX_API_URL,
        }
    }

    async fn get<T>(&self, path: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .reqwest_client
            .get(format!("{}{}", self.url, path))
            .header(reqwest::header::USER_AGENT, "coinbase_client")
            .send()
            .await?;
        deserialize_response(response).await
    }

    /// get a list of available currency pairs for trading
    pub async fn get_products(&self) -> Result<Vec<Product>, Error> {
        let products: Vec<Product> = self.get("/products").await?;
        Ok(products)
    }

    /// get market data for a specific currency pair.
    pub async fn get_product(&self, id: &str) -> Result<Product, Error> {
        let product: Product = self.get(&format!("/products/{}", id)).await?;
        Ok(product)
    }

    async fn get_order_book(
        &self,
        id: &str,
        level: OrderLevel,
    ) -> Result<OrderBook<BookEntry>, Error> {
        let book: OrderBook<BookEntry> = self
            .get(&format!("/products/{}/book?level={}", id, level as u8))
            .await?;
        Ok(book)
    }

    /// get only the best bid and ask
    pub async fn get_product_order_book(&self, id: &str) -> Result<OrderBook<BookEntry>, Error> {
        Ok(self.get_order_book(id, OrderLevel::One).await?)
    }

    /// get top 50 bids and asks
    pub async fn get_product_order_book_top50(
        &self,
        id: &str,
    ) -> Result<OrderBook<BookEntry>, Error> {
        Ok(self.get_order_book(id, OrderLevel::Two).await?)
    }

    /// get Full order book
    pub async fn get_product_order_book_all(
        &self,
        id: &str,
    ) -> Result<OrderBook<FullBookEntry>, Error> {
        let book: OrderBook<FullBookEntry> =
            self.get(&format!("/products/{}/book?level=3", id)).await?;
        Ok(book)
    }

    /// get snapshot information about the last trade (tick), best bid/ask and 24h volume.
    pub async fn get_product_ticker(&self, id: &str) -> Result<Ticker, Error> {
        let ticker = self.get(&format!("/products/{}/ticker", id)).await?;
        Ok(ticker)
    }

    /// get the latest trades for a product.
    ///<br>
    ///<br>
    /// **PARAMETERS**
    /// <br>
    ///before: Request page before (newer) this pagination id.
    ///<br>
    ///after: Request page after (older) this pagination id.
    ///<br>
    ///limit: Number of results per request. Maximum 1000. (default 1000)
    ///<br>
    pub async fn get_product_trades(
        &self,
        id: &str,
        before_pagination_id: Option<u64>,
        after_pagination_id: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<Trade>, Error> {
        let mut path = format!("/products/{}/trades", id);
        let mut appended = false;
        if let Some(n) = before_pagination_id {
            appended = true;
            path.push_str(&format!("?before={}", n))
        }
        if let Some(n) = after_pagination_id {
            if appended {
                path.push_str(&format!("&after={}", n))
            } else {
                appended = true;
                path.push_str(&format!("?after={}", n))
            }
        }
        if let Some(mut n) = limit {
            if n > 1000 {
                n = 1000;
            }
            if appended {
                path.push_str(&format!("&limit={}", n))
            } else {
                path.push_str(&format!("?limit={}", n))
            }
        }
        let trades: Vec<Trade> = self.get(&path).await?;
        Ok(trades)
    }

    /// get historic rates for a product
    /// <br>
    /// <br>
    /// **PARAMETERS**
    /// <br>
    /// start: Start time in ISO 8601
    ///<br>
    /// end: End time in ISO 8601
    ///<br>
    /// granularity: Desired timeslice in seconds
    pub async fn get_product_historic_rates(
        &self,
        id: &str,
        start: Option<&str>,
        end: Option<&str>,
        granularity: Option<Granularity>,
    ) -> Result<Vec<HistoricRate>, Error> {
        let mut appended = false;
        let mut path = format!("/products/{}/candles", id);
        if let Some(n) = start {
            appended = true;
            path.push_str(&format!("?start={}", n));
        }

        if let Some(n) = end {
            if appended {
                path.push_str(&format!("&end={}", n));
            } else {
                path.push_str(&format!("?end={}", n));
            }
        }
        if let Some(n) = granularity {
            if appended {
                path.push_str(&format!("&granularity={}", n as u32));
            } else {
                path.push_str(&format!("?granularity={}", n as u32));
            }
        }
        let rates: Vec<HistoricRate> = self.get(&path).await?;
        Ok(rates)
    }

    /// get 24 hr stats for the product
    pub async fn get_product_24hr_stats(&self, id: &str) -> Result<TwentyFourHourStats, Error> {
        let stats: TwentyFourHourStats = self.get(&format!("/products/{}/stats", id)).await?;
        Ok(stats)
    }

    /// get known currencies
    pub async fn get_currencies(&self) -> Result<Vec<Currency>, Error> {
        let currencies: Vec<Currency> = self.get("/currencies").await?;
        Ok(currencies)
    }

    /// get the currency for specified id
    pub async fn get_currency(&self, id: &str) -> Result<Currency, Error> {
        let currency: Currency = self.get(&format!("/currencies/{}", id)).await?;
        Ok(currency)
    }

    /// get the API server time
    pub async fn get_time(&self) -> Result<Time, Error> {
        let time: Time = self.get("/time").await?;
        Ok(time)
    }
}

/// a structure that represents a product
#[derive(serde::Deserialize, Debug)]
pub struct Product {
    pub id: String,
    pub display_name: String,
    pub base_currency: String,
    pub quote_currency: String,
    /// Coinbase api returns floats as strings, im using this to convert them to floats
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub base_increment: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub quote_increment: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub base_min_size: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub base_max_size: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub min_market_funds: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub max_market_funds: f64,
    pub status: String,
    pub status_message: String,
    pub cancel_only: bool,
    pub limit_only: bool,
    pub post_only: bool,
    pub trading_disabled: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct BookEntry {
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub size: f64,
    pub num_orders: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct FullBookEntry {
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub size: f64,
    pub order_id: String,
}
/// a structure that represents the trade list of open orders for a product
#[derive(serde::Deserialize, Debug)]
pub struct OrderBook<T> {
    pub bids: Vec<T>,
    pub asks: Vec<T>,
    pub sequence: u64,
}

/// a structure that represents a trade
#[derive(serde::Deserialize, Debug)]
pub struct Trade {
    #[serde(deserialize_with = "deserialize_to_date")]
    pub time: DateTime<Utc>,
    pub trade_id: u64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub size: f64,
    pub side: String,
}

/// a structure that represents latest trades for a product
#[derive(serde::Deserialize, Debug)]
pub struct Ticker {
    pub trade_id: u64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub size: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub bid: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub ask: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub volume: f64,
    #[serde(deserialize_with = "deserialize_to_date")]
    pub time: DateTime<Utc>,
}

/// a structure that represents rates for a product
#[derive(serde::Deserialize, Debug)]
pub struct HistoricRate {
    pub time: f64,
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}

/// a structure that represents 24 hr stats for a product
#[derive(serde::Deserialize, Debug)]
pub struct TwentyFourHourStats {
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub open: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub high: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub low: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub volume: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub last: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub volume_30day: f64,
}

// some field are ompited when a single currency is returned hence the Options enum
/// a structure that represents a currency
#[derive(serde::Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub min_size: f64,
    pub status: String,
    pub message: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub max_precision: f64,
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

/// a structure that represents the API server time.
#[derive(serde::Deserialize, Debug)]
pub struct Time {
    #[serde(deserialize_with = "deserialize_to_date")]
    pub iso: DateTime<Utc>,
    pub epoch: f64,
}

enum OrderLevel {
    One = 1,
    Two = 2,
}

/// desired timeslice in seconds {60, 300, 900, 3600, 21600, 86400}
pub enum Granularity {
    OneMinute = 60,
    FiveMinutes = 300,
    FifteenMinutes = 900,
    OneHour = 3600,
    SixHours = 21600,
    OneDay = 86400,
}
