use super::error::{Error, ErrorKind};
use super::json;
use futures;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

const COINBASE_API_URL: &str = "https://api.pro.coinbase.com";

pub enum OrderLevel {
    one,
    two,
    three,
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

    async fn get(&self, url: &str) -> Result<reqwest::Response, Error> {
        let res = self
            .reqwest_client
            .get(url)
            .header(reqwest::header::USER_AGENT, "rusty-coin")
            .send()
            .await?;
        if !res.status().is_success() {
            return Err(Error::new(ErrorKind::Status));
        }

        Ok(res)
    }

    pub async fn get_products(&self) -> Result<Vec<json::Product>, Error> {
        let url = format!("{}/products", COINBASE_API_URL);
        let products: Vec<json::Product> = self.get(&url).await?.json().await?;
        Ok(products)
    }

    pub async fn get_product(&self, id: &str) -> Result<json::Product, Error> {
        let url = format!("{}/products/{}", COINBASE_API_URL, id);
        let product: json::Product = self.get(&url).await?.json().await?;
        Ok(product)
    }

    // level 1 Only the best bid and ask
    // level 2 Top 50 bids and asks (aggregated)
    // level 3 Full order book (non aggregated)
    pub async fn get_product_order_book(
        &self,
        id: &str,
        level: OrderLevel,
    ) -> Result<json::OrderBook, Error> {
        let level = match level {
            OrderLevel::one => "1",
            OrderLevel::two => "2",
            OrderLevel::three => "3",
        };
        let url = format!("{}/products/{}/book?level={}", COINBASE_API_URL, id, level);
        let book: json::OrderBook = self.get(&url).await?.json().await?;
        Ok(book)
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

    async fn test_get_product_order_book() {
        let client = PublicClient::new();
        let future1 = client.get_product_order_book("MIR-EUR", OrderLevel::one);
        let future2 = client.get_product_order_book("MIR-EUR", OrderLevel::two);
        let future3 = client.get_product_order_book("MIR-EUR", OrderLevel::three);
        let json1 = futures::executor::block_on(future1).unwrap();
        let json2 = futures::executor::block_on(future2).unwrap();
        let json3 = futures::executor::block_on(future3).unwrap();
    }
}
