use futures;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
const COINBASE_API_URL: &str = "https://api.pro.coinbase.com";


#[derive(Serialize, Deserialize, Debug)]
pub struct Product {
    id: String,
    display_name: String,
    base_currency: String,
    quote_currency: String,
    base_increment: String,
    quote_increment: String,
    base_min_size: String,
    base_max_size: String,
    min_market_funds: String,
    max_market_funds: String,
    status: String,
    status_message: String,
    cancel_only: bool,
    limit_only: bool,
    post_only: bool,
    trading_disabled: bool,
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

    async fn get(&self, url: String) -> Result<reqwest::Response, reqwest::Error> {
        let res = self
            .reqwest_client
            .get(url)
            .header(reqwest::header::USER_AGENT, "rusty-coin")
            .send()
            .await?;
        if !res.status().is_success() {
        
        }

        
        Ok(res)
    }

    pub async fn get_products(&self) -> Result<Vec<Product>, reqwest::Error>  {
        let url = format!("{}/products", COINBASE_API_URL);
        let products: Vec<Product> = self.get(url).await?.json().await?;
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
    async fn test_get_products() {
        let client = PublicClient::new();
        let future = client.get_products();
        let res = futures::executor::block_on(future).unwrap();
        for p in res.iter() {
            println!("{:?}", p.id)
        }
        assert!(res.len() > 0);
    }
}
