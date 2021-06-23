use super::{
    deserialize_response, deserialize_to_date, COINBASE_API_URL, COINBASE_SANDBOX_API_URL,
};
use crate::{configure_pagination, error::Error};
use chrono::{DateTime, Utc};
use reqwest;
use serde;

/// `PublicClient provides public market data
pub struct PublicClient {
    reqwest_client: reqwest::Client,
    url: &'static str,
}

impl PublicClient {
    async fn get_paginated<T>(
        &self,
        path: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let params = configure_pagination(before, after, limit);
        self.get(&format!("{}{}", path, params)).await
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

    /// Creates a `PublicClient`
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// ~~~~
    pub fn new() -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            url: COINBASE_API_URL,
        }
    }

    /// Creates a `PublicClient` to be used with the coinbase pro sandbox API
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// ~~~~
    pub fn new_sandbox() -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            url: COINBASE_SANDBOX_API_URL,
        }
    }

    /// Get a list of available currency pairs for trading
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-products)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let products = client.get_products().await.unwrap();
    /// ~~~~
    pub async fn get_products(&self) -> Result<Vec<Product>, Error> {
        let products: Vec<Product> = self.get("/products").await?;
        Ok(products)
    }

    /// Get market data for a specific currency pair.
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-single-product)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let product = client.get_product("BTC-USD").await.unwrap();
    /// ~~~~
    pub async fn get_product(&self, id: &str) -> Result<Product, Error> {
        let product: Product = self.get(&format!("/products/{}", id)).await?;
        Ok(product)
    }

    // Get a list of open orders for a product
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

    /// Get a list of open orders for a product
    /// <br>
    /// Gets only the best bid and ask
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-product-order-book)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let order_book = client.get_product_order_book("BTC-USD").await.unwrap();
    /// ~~~~
    pub async fn get_product_order_book(&self, id: &str) -> Result<OrderBook<BookEntry>, Error> {
        Ok(self.get_order_book(id, OrderLevel::One).await?)
    }

    /// Get a list of open orders for a product
    /// <br>
    /// Gets top 50 bids and asks
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-product-order-book)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let order_book = client
    ///     .get_product_order_book_top50("BTC-USD")
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_product_order_book_top50(
        &self,
        id: &str,
    ) -> Result<OrderBook<BookEntry>, Error> {
        Ok(self.get_order_book(id, OrderLevel::Two).await?)
    }

    /// Get a list of open orders for a product
    /// <br>
    /// Get Full order book
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-product-order-book)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let order_book = client.get_product_order_book_all("BTC-USD").await.unwrap();
    /// ~~~~
    pub async fn get_product_order_book_all(
        &self,
        id: &str,
    ) -> Result<OrderBook<FullBookEntry>, Error> {
        let book: OrderBook<FullBookEntry> =
            self.get(&format!("/products/{}/book?level=3", id)).await?;
        Ok(book)
    }

    /// Get snapshot information about the last trade (tick), best bid/ask and 24h volume.
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-product-ticker)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let ticker = client
    ///     .get_product_ticker("BTC-USD", Some("30902419"), None, None)
    ///     .await
    ///     .unwrap();    
    /// ~~~~
    pub async fn get_product_ticker(
        &self,
        id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Ticker, Error> {
        let ticker = self
            .get_paginated(&format!("/products/{}/ticker?", id), before, after, limit)
            .await?;
        Ok(ticker)
    }

    /// Get the latest trades for a product.
    ///<br>
    ///<br>
    /// **PARAMETERS**
    /// <br>
    ///before: Request page before (newer) this pagination id.
    ///<br>
    ///after: Request page after (older) this pagination id.
    ///<br>
    ///limit: Number of results per request. Maximum 1000. (default 1000)
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-trades)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let trades = client
    ///     .get_product_trades("BTC-USD", None, Some("30898635"), Some(100))
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_product_trades(
        &self,
        id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<Trade>, Error> {
        let trades: Vec<Trade> = self
            .get_paginated(&format!("/products/{}/trades?", id), before, after, limit)
            .await?;
        Ok(trades)
    }

    /// get historic rates for a product
    /// <br>
    /// <br>
    /// **PARAMETERS**
    /// <br>
    /// *start*: Start time in ISO 8601
    /// <br>
    /// *end*: End time in ISO 8601
    /// <br>
    /// *granularity*: Desired timeslice in seconds
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-historic-rates)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let historical_rates = client
    ///     .get_product_historic_rates("BTC-USD", None, None, Some(Granularity::OneMinute))
    ///     .await
    ///     .unwrap();
    /// ~~~~
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

    /// Get 24 hr stats for the product
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-24hr-stats)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let twenty_four_hour_stats = client.get_product_24hr_stats("BTC-USD").await.unwrap();
    /// ~~~~
    pub async fn get_product_24hr_stats(&self, id: &str) -> Result<TwentyFourHourStats, Error> {
        let stats: TwentyFourHourStats = self.get(&format!("/products/{}/stats", id)).await?;
        Ok(stats)
    }

    /// Get known currencies
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-currencies)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let currencies = client.get_currencies().await.unwrap();
    /// ~~~~
    pub async fn get_currencies(&self) -> Result<Vec<Currency>, Error> {
        let currencies: Vec<Currency> = self.get("/currencies").await?;
        Ok(currencies)
    }

    /// Get the currency for specified id
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#get-a-currency)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let currency = client.get_currency("LINK").await.unwrap();
    /// ~~~~
    pub async fn get_currency(&self, id: &str) -> Result<Currency, Error> {
        Ok(self.get(&format!("/currencies/{}", id)).await?)
    }

    /// Get the API server time
    /// <br>
    /// [api docs](https://docs.pro.coinbase.com/#time)
    /// <br>
    /// ~~~~
    /// let client = PublicClient::new_sandbox("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let time = client.get_time().await.unwrap();
    /// ~~~~
    pub async fn get_time(&self) -> Result<Time, Error> {
        let time: Time = self.get("/time").await?;
        Ok(time)
    }
}

/// A structure that represents a product
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

/// A structure that represents the trade list of open orders for a product
#[derive(serde::Deserialize, Debug)]
pub struct OrderBook<T> {
    pub bids: Vec<T>,
    pub asks: Vec<T>,
    pub sequence: u64,
}

/// A structure that represents a trade
#[derive(serde::Deserialize, Debug)]
pub struct Trade {
    #[serde(deserialize_with = "deserialize_to_date")]
    pub time: DateTime<Utc>,
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub side: String,
}

/// A structure that represents latest trades for a product
#[derive(serde::Deserialize, Debug)]
pub struct Ticker {
    pub trade_id: u64,
    pub price: String,
    pub size: String,
    pub bid: String,
    pub ask: String,
    pub volume: String,
    #[serde(deserialize_with = "deserialize_to_date")]
    pub time: DateTime<Utc>,
}

/// A structure that represents rates for a product
#[derive(serde::Deserialize, Debug)]
pub struct HistoricRate {
    pub time: u64,
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}

/// A structure that represents 24 hr stats for a product
#[derive(serde::Deserialize, Debug)]
pub struct TwentyFourHourStats {
    pub open: String,
    pub high: String,
    pub low: String,
    pub volume: String,
    pub last: String,
    pub volume_30day: String,
}

/// A structure that represents a currency
#[derive(serde::Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    pub min_size: String,
    pub status: String,
    pub message: Option<String>,
    pub max_precision: String,
    pub convertible_to: Option<Vec<String>>,
    pub details: CurrencyDetails,
}

#[derive(serde::Deserialize, Debug)]
pub struct CurrencyDetails {
    pub r#type: String, // use raw identifier to allow reserved keyword
    pub symbol: String,
    pub network_confirmations: u64,
    pub sort_order: u64,
    pub crypto_address_link: String,
    pub crypto_transaction_link: String,
    pub push_payment_methods: Vec<String>,
    pub group_types: Option<Vec<String>>,
    pub display_name: Option<String>,
    pub processing_time_seconds: Option<f64>,
    pub min_withdrawal_amount: f64,
    pub max_withdrawal_amount: f64,
}

/// A structure that represents the API server time.
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

/// Desired timeslice in seconds {60, 300, 900, 3600, 21600, 86400}
pub enum Granularity {
    OneMinute = 60,
    FiveMinutes = 300,
    FifteenMinutes = 900,
    OneHour = 3600,
    SixHours = 21600,
    OneDay = 86400,
}
