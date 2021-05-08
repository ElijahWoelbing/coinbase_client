use super::error::{Error, ErrorKind};
use super::json;
use futures;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

const COINBASE_API_URL: &str = "https://api.pro.coinbase.com";

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
            return Err(Error::new(ErrorKind::HttpError));
        }

        Ok(res)
    }

    pub async fn get_products(&self) -> Result<Vec<json::Product>, Error> {
        let url = format!("{}/products", COINBASE_API_URL);
        let products: Vec<json::Product> = self.get(&url).await?.json().await?;
        Ok(products)
    }
}

impl Default for PublicClient {
    fn default() -> Self {
        Self::new()
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
}
