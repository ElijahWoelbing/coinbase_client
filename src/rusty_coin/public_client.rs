use super::error::{Error, ErrorKind};
use super::json::*;
use futures;
use reqwest;
use serde::de::DeserializeOwned;

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

    async fn get_and_deserialize<T>(&self, url: &str) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let mut res = self
            .reqwest_client
            .get(url)
            .header(reqwest::header::USER_AGENT, "rusty-coin")
            .send()
            .await?;
        if !res.status().is_success() {
            // return Err(Error::new(ErrorKind::Status));
        }
        let json = res.json().await?;

        Ok(json)
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

    async fn order_book(
        &self,
        id: &str,
        level: OrderLevel,
    ) -> Result<OrderBook<BookEntry>, reqwest::Error> {
        let url = format!(
            "{}/products/{}/book?level={}",
            COINBASE_API_URL, id, level as u8
        );
        let book: OrderBook<BookEntry> = self.get_and_deserialize(&url).await?;
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
        let book: OrderBook<FullBookEntry> = self.get_and_deserialize(&url).await?;
        Ok(book)
    }

    pub async fn get_product_ticker(&self, id: &str) -> Result<Ticker, reqwest::Error> {
        let url = format!("{}/products/{}/ticker", COINBASE_API_URL, id);
        let ticker = self.get_and_deserialize(&url).await?;
        Ok(ticker)
    }

    pub async fn get_product_trades(&self, id: &str) -> Result<Vec<Trade>, reqwest::Error> {
        let url = format!("{}/products/{}/trades", COINBASE_API_URL, id);
        let trades: Vec<Trade> = self.get_and_deserialize(&url).await?;
        Ok(trades)
    }

    pub async fn get_product_historic_rates(
        &self,
        id: &str,
    ) -> Result<Vec<HistoricRate>, reqwest::Error> {
        let url = format!("{}/products/{}/candles", COINBASE_API_URL, id);
        let rates: Vec<HistoricRate> = self.get_and_deserialize(&url).await?;
        Ok(rates)
    }

    pub async fn get_product_24hr_stats(
        &self,
        id: &str,
    ) -> Result<TwentyFourHourStats, reqwest::Error> {
        let url = format!("{}/products/{}/stats", COINBASE_API_URL, id);
        let stats: TwentyFourHourStats = self.get_and_deserialize(&url).await?;
        Ok(stats)
    }

    pub async fn get_currencies(&self) -> Result<Vec<Currency>, reqwest::Error> {
        let url = format!("{}/currencies", COINBASE_API_URL);
        let currencies: Vec<Currency> = self.get_and_deserialize(&url).await?;
        Ok(currencies)
    }

    pub async fn get_currency(&self, id: &str) -> Result<Currency, reqwest::Error> {
        let url = format!("{}/currencies/{}", COINBASE_API_URL, id);
        let currency: Currency = self.get_and_deserialize(&url).await?;
        Ok(currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_and_deserialize() {
        let client = PublicClient::new();
        let future = client.get_and_deserialize("https://www.rust-lang.org");
        let res = futures::executor::block_on(future).unwrap();
    }
}
