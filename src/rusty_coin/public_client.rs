use super::error::{Error, ErrorKind};
use super::json::*;
use futures;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

const COINBASE_API_URL: &str = "https://api.pro.coinbase.com";

enum OrderLevel {
    One = 1,
    Two = 2,
    Three = 3,
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

    async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let res = self
            .reqwest_client
            .get(url)
            .header(reqwest::header::USER_AGENT, "rusty-coin")
            .send()
            .await?;
        if !res.status().is_success() {
            // return Err(Error::new(ErrorKind::Status));
        }

        Ok(res)
    }

    pub async fn get_products(&self) -> Result<Vec<Product>, Error> {
        let url = format!("{}/products", COINBASE_API_URL);
        let products: Vec<Product> = self.get(&url).await?.json().await?;
        Ok(products)
    }

    pub async fn get_product(&self, id: &str) -> Result<Product, Error> {
        let url = format!("{}/products/{}", COINBASE_API_URL, id);
        let product: Product = self.get(&url).await?.json().await?;
        Ok(product)
    }

    async fn order_book(
        &self,
        id: &str,
        level: OrderLevel,
    ) -> Result<OrderBook<BookEntry>, reqwest::Error> {
        let url = format!(
            "{}/products/{}/book?level={}",
            COINBASE_API_URL, id, level as u8
        );
        let book: OrderBook<BookEntry> = self.get(&url).await?.json().await?;
        Ok(book)
    }

    // level 1 Only the best bid and ask
    pub async fn get_product_order_book(
        &self,
        id: &str,
    ) -> Result<OrderBook<BookEntry>, reqwest::Error> {
        Ok(self.order_book(id, OrderLevel::One).await?)
    }

    // level 2 Top 50 bids and asks (aggregated)
    pub async fn get_product_order_book_top50(
        &self,
        id: &str,
    ) -> Result<OrderBook<BookEntry>, reqwest::Error> {
        Ok(self.order_book(id, OrderLevel::Two).await?)
    }

    // level 3 Full order book (non aggregated)
    pub async fn get_product_order_book_all(
        &self,
        id: &str,
    ) -> Result<OrderBook<FullBookEntry>, reqwest::Error> {
        let url = format!("{}/products/{}/book?level=3", COINBASE_API_URL, id);
        let book: OrderBook<FullBookEntry> = self.get(&url).await?.json().await?;
        Ok(book)
    }

    async fn get_product_ticker(&self, id: &str) -> Result<Ticker, reqwest::Error> {
        let url = format!("{}/products/{}/ticker", COINBASE_API_URL, id);
        let ticker = self.get(&url).await?.json().await?;
        Ok(ticker)
    }

    async fn get_product_trades(&self, id: &str) -> Result<Vec<Trade>, reqwest::Error> {
        let url = format!("{}/products/{}/trades", COINBASE_API_URL, id);
        let trades: Vec<Trade> = self.get(&url).await?.json().await?;
        Ok(trades)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get() {
        let client = PublicClient::new();
        let future = client.get("https://www.rust-lang.org");
        let res = futures::executor::block_on(future).unwrap();
        assert!(res.status().is_success());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_products() {
        let client = PublicClient::new();
        let future = client.get_products();
        let json = futures::executor::block_on(future).unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_product() {
        let client = PublicClient::new();
        let future = client.get_product("MIR-EUR");
        let json = futures::executor::block_on(future).unwrap();
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_product_order_book_all() {
        let client = PublicClient::new();
        let future = client.get_product_order_book_all("MIR-EUR");
        let json = futures::executor::block_on(future).unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_product_order_book_top50() {
        let client = PublicClient::new();
        let future = client.get_product_order_book_top50("MIR-EUR");
        let json = futures::executor::block_on(future).unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_product_order_book() {
        let client = PublicClient::new();
        let future = client.get_product_order_book("MIR-EUR");
        let json = futures::executor::block_on(future).unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_product_ticker() {
        let client = PublicClient::new();
        let future = client.get_product_ticker("MIR-EUR");
        let json = futures::executor::block_on(future).unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_product_trades() {
        let client = PublicClient::new();
        let future = client.get_product_trades("MIR-EUR");
        let json = futures::executor::block_on(future).unwrap();
    }
}
