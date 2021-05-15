use super::error::Error;
use super::json::*;
use reqwest;
use serde::de::DeserializeOwned;

const COINBASE_API_URL: &str = "https://api.pro.coinbase.com";

enum OrderLevel {
    One = 1,
    Two = 2,
}

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

    async fn get_and_deserialize<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let res = self
            .reqwest_client
            .get(url)
            .header(reqwest::header::USER_AGENT, "rusty-coin")
            .send()
            .await?
            .json()
            .await?;
        Ok(res)
    }

    pub async fn get_products(&self) -> Result<Vec<Product>, Error> {
        let url = format!("{}/products", COINBASE_API_URL);
        let products: Vec<Product> = self.get_and_deserialize(&url).await?;
        Ok(products)
    }

    pub async fn get_product(&self, id: &str) -> Result<Product, Error> {
        let url = format!("{}/products/{}", COINBASE_API_URL, id);
        let product: Product = self.get_and_deserialize(&url).await?;
        Ok(product)
    }

    async fn get_order_book(
        &self,
        id: &str,
        level: OrderLevel,
    ) -> Result<OrderBook<BookEntry>, Error> {
        let url = format!(
            "{}/products/{}/book?level={}",
            COINBASE_API_URL, id, level as u8
        );
        let book: OrderBook<BookEntry> = self.get_and_deserialize(&url).await?;
        Ok(book)
    }

    // level 1 Only the best bid and ask
    pub async fn get_product_order_book(&self, id: &str) -> Result<OrderBook<BookEntry>, Error> {
        Ok(self.get_order_book(id, OrderLevel::One).await?)
    }

    // level 2 Top 50 bids and asks (aggregated)
    pub async fn get_product_order_book_top50(
        &self,
        id: &str,
    ) -> Result<OrderBook<BookEntry>, Error> {
        Ok(self.get_order_book(id, OrderLevel::Two).await?)
    }

    // level 3 Full order book (non aggregated)
    pub async fn get_product_order_book_all(
        &self,
        id: &str,
    ) -> Result<OrderBook<FullBookEntry>, Error> {
        let url = format!("{}/products/{}/book?level=3", COINBASE_API_URL, id);
        let book: OrderBook<FullBookEntry> = self.get_and_deserialize(&url).await?;
        Ok(book)
    }

    pub async fn get_product_ticker(&self, id: &str) -> Result<Ticker, Error> {
        let url = format!("{}/products/{}/ticker", COINBASE_API_URL, id);
        let ticker = self.get_and_deserialize(&url).await?;
        Ok(ticker)
    }

    pub async fn get_product_trades(&self, id: &str) -> Result<Vec<Trade>, Error> {
        let url = format!("{}/products/{}/trades", COINBASE_API_URL, id);
        let trades: Vec<Trade> = self.get_and_deserialize(&url).await?;
        Ok(trades)
    }

    pub async fn get_product_historic_rates(&self, id: &str) -> Result<Vec<HistoricRate>, Error> {
        let url = format!("{}/products/{}/candles", COINBASE_API_URL, id);
        let rates: Vec<HistoricRate> = self.get_and_deserialize(&url).await?;
        Ok(rates)
    }

    pub async fn get_product_24hr_stats(&self, id: &str) -> Result<TwentyFourHourStats, Error> {
        let url = format!("{}/products/{}/stats", COINBASE_API_URL, id);
        let stats: TwentyFourHourStats = self.get_and_deserialize(&url).await?;
        Ok(stats)
    }

    pub async fn get_currencies(&self) -> Result<Vec<Currency>, Error> {
        let url = format!("{}/currencies", COINBASE_API_URL);
        let currencies: Vec<Currency> = self.get_and_deserialize(&url).await?;
        Ok(currencies)
    }

    pub async fn get_currency(&self, id: &str) -> Result<Currency, Error> {
        let url = format!("{}/currencies/{}", COINBASE_API_URL, id);
        let currency: Currency = self.get_and_deserialize(&url).await?;
        Ok(currency)
    }

    pub async fn get_time(&self) -> Result<Time,Error>{
        let url = format!("{}/time", COINBASE_API_URL);
        let time: Time = self.get_and_deserialize(&url).await?;
        Ok(time)
    }
}
