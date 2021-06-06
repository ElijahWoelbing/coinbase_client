use super::{deserialize_f64, deserialize_response, COINBASE_API_URL};
use crate::error::Error;
use reqwest;
use serde;

enum OrderLevel {
    One = 1,
    Two = 2,
}

/// `PublicClient provides public market data
pub struct PublicClient {
    reqwest_client: reqwest::Client,
}

impl PublicClient {
    pub fn new() -> Self {
        Self {
            // create client for making http reqwests
            reqwest_client: reqwest::Client::new(),
        }
    }

    async fn get<T>(&self, url: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self
            .reqwest_client
            .get(url)
            .header(reqwest::header::USER_AGENT, "rusty-coin")
            .send()
            .await?;

        deserialize_response(response).await
    }

    /// get a list of available currency pairs for trading
    pub async fn get_products(&self) -> Result<Vec<Product>, Error> {
        let url = format!("{}/products", COINBASE_API_URL);
        let products: Vec<Product> = self.get(&url).await?;
        Ok(products)
    }

    /// get market data for a specific currency pair.
    pub async fn get_product(&self, id: &str) -> Result<Product, Error> {
        let product: Product = self
            .get(&format!("{}/products/{}", COINBASE_API_URL, id))
            .await?;
        Ok(product)
    }

    async fn get_order_book(
        &self,
        id: &str,
        level: OrderLevel,
    ) -> Result<OrderBook<BookEntry>, Error> {
        let book: OrderBook<BookEntry> = self
            .get(&format!(
                "{}/products/{}/book?level={}",
                COINBASE_API_URL, id, level as u8
            ))
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
        let book: OrderBook<FullBookEntry> = self
            .get(&format!(
                "{}/products/{}/book?level=3",
                COINBASE_API_URL, id
            ))
            .await?;
        Ok(book)
    }

    /// get snapshot information about the last trade (tick), best bid/ask and 24h volume.
    pub async fn get_product_ticker(&self, id: &str) -> Result<Ticker, Error> {
        let ticker = self
            .get(&format!("{}/products/{}/ticker", COINBASE_API_URL, id))
            .await?;
        Ok(ticker)
    }

    /// get the latest trades for a product.
    pub async fn get_product_trades(&self, id: &str) -> Result<Vec<Trade>, Error> {
        let trades: Vec<Trade> = self
            .get(&format!("{}/products/{}/trades", COINBASE_API_URL, id))
            .await?;
        Ok(trades)
    }

    /// get historic rates for a product
    pub async fn get_product_historic_rates(&self, id: &str) -> Result<Vec<HistoricRate>, Error> {
        let rates: Vec<HistoricRate> = self
            .get(&format!("{}/products/{}/candles", COINBASE_API_URL, id))
            .await?;
        Ok(rates)
    }

    /// get 24 hr stats for the product
    pub async fn get_product_24hr_stats(&self, id: &str) -> Result<TwentyFourHourStats, Error> {
        let stats: TwentyFourHourStats = self
            .get(&format!("{}/products/{}/stats", COINBASE_API_URL, id))
            .await?;
        Ok(stats)
    }

    /// get known currencies
    pub async fn get_currencies(&self) -> Result<Vec<Currency>, Error> {
        let currencies: Vec<Currency> = self
            .get(&format!("{}/currencies", COINBASE_API_URL))
            .await?;
        Ok(currencies)
    }

    /// get the currency for specified id
    pub async fn get_currency(&self, id: &str) -> Result<Currency, Error> {
        let currency: Currency = self
            .get(&format!("{}/currencies/{}", COINBASE_API_URL, id))
            .await?;
        Ok(currency)
    }

    /// get the API server time
    pub async fn get_time(&self) -> Result<Time, Error> {
        let time: Time = self.get(&format!("{}/time", COINBASE_API_URL)).await?;
        Ok(time)
    }
}

/// data for a specific currency
#[derive(serde::Deserialize, Debug)]
pub struct Product {
    pub id: String,
    pub display_name: String,
    pub base_currency: String,
    pub quote_currency: String,
    /// Coinbase api returns floats as strings, im using this to convert them to floats
    #[serde(deserialize_with = "deserialize_f64")]
    pub base_increment: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub quote_increment: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub base_min_size: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub base_max_size: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub min_market_funds: f64,
    #[serde(deserialize_with = "deserialize_f64")]
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
    #[serde(deserialize_with = "deserialize_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub size: f64,
    pub num_orders: u64,
}

#[derive(serde::Deserialize, Debug)]
pub struct FullBookEntry {
    #[serde(deserialize_with = "deserialize_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub size: f64,
    pub order_id: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct OrderBook<T> {
    pub bids: Vec<T>,
    pub asks: Vec<T>,
    pub sequence: u64,
}

/// a structure that represents a trade
#[derive(serde::Deserialize, Debug)]
pub struct Trade {
    pub time: String,
    pub trade_id: u64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub size: f64,
    pub side: String,
}

/// a structure that represents latest trades for a produc
#[derive(serde::Deserialize, Debug)]
pub struct Ticker {
    pub trade_id: u64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub size: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub bid: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub ask: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub volume: f64,
    pub time: String,
}

/// a structure that represents rates for a product
#[derive(serde::Deserialize, Debug)]
pub struct HistoricRate {
    pub time: u64,
    pub low: f64,
    pub high: f64,
    pub open: f64,
    pub close: f64,
    pub volume: f64,
}

/// a structure that represents 24 hr stats for a product
#[derive(serde::Deserialize, Debug)]
pub struct TwentyFourHourStats {
    #[serde(deserialize_with = "deserialize_f64")]
    pub open: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub high: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub low: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub volume: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub last: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub volume_30day: f64,
}

// some field are ompited when a single currency is returned hence the Options enum
/// a structure that represents a currency
#[derive(serde::Deserialize, Debug)]
pub struct Currency {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_f64")]
    pub min_size: f64,
    pub status: String,
    pub message: String,
    #[serde(deserialize_with = "deserialize_f64")]
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

#[derive(serde::Deserialize, Debug)]
pub struct Time {
    pub iso: String,
    pub epoch: f64,
}
