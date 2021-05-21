use super::error::{Error, ErrorKind};
use super::json::*;
use base64;
use chrono::format::format;
use core::f64;
use crypto::{self, mac::Mac};
use futures;
use reqwest::{self, header::HeaderMap};
use serde::{de::DeserializeOwned, ser, Deserialize, Serialize, Serializer};
use std::{
    io::Bytes,
    time::{SystemTime, SystemTimeError},
};
use uuid::{self, Uuid};

const COINBASE_API_URL: &str = "https://api-public.sandbox.pro.coinbase.com";

pub struct PrivateClient {
    reqwest_client: reqwest::Client,
    secret: String,
    passphrase: String,
    key: String,
}

impl PrivateClient {
    pub fn new(secret: String, passphrase: String, key: String) -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            secret, // shared secret
            key,
            passphrase,
        }
    }

    async fn get_and_deserialize<T>(&self, path: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let headers = self.access_headers(path, None, "GET");
        let res = self
            .reqwest_client
            .get(format!("{}{}", COINBASE_API_URL, path))
            .headers(headers)
            .send()
            .await?;
        let status = res.status();
        if !status.is_success() {
            println!("{}", res.text().await?);
            return Err(Error::new(ErrorKind::Status(status)));
        }
        let json = res.json().await?;
        Ok(json)
    }

    fn get_current_timestamp() -> Result<String, SystemTimeError> {
        Ok(SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            .to_string())
    }

    fn access_headers(
        &self,
        url: &str,
        body: Option<&str>,
        meathod: &str,
    ) -> reqwest::header::HeaderMap {
        let timestamp = PrivateClient::get_current_timestamp().unwrap();
        let signature = self.sign_message(url, body, &timestamp, meathod);
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_str("coinbase-client")
                .expect("invalid user agent value"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-key"),
            reqwest::header::HeaderValue::from_str(&self.key)
                .expect("invalid user cb-access-key value"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-sign"),
            reqwest::header::HeaderValue::from_str(&signature)
                .expect("invalid cb-access-sign value"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-timestamp"),
            reqwest::header::HeaderValue::from_str(&timestamp)
                .expect("invalid user cb-access-timestamp value"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("cb-access-passphrase"),
            reqwest::header::HeaderValue::from_str(&self.passphrase)
                .expect("invalid user cb-access-passphrase value"),
        );

        headers
    }

    pub fn sign_message(
        &self,
        url: &str,
        body: Option<&str>,
        timestamp: &str,
        meathod: &str,
    ) -> String {
        let mut prehash = String::new();
        // omit body if not supplied
        match body {
            Some(body) => {
                prehash.push_str(&timestamp);
                prehash.push_str(&meathod);
                prehash.push_str(&url);
                prehash.push_str(&body);
            }
            None => {
                prehash.push_str(&timestamp);
                prehash.push_str(&meathod);
                prehash.push_str(&url);
            }
        }
        // decode your coinbase api secret
        let decoded_secret = base64::decode(&self.secret)
            .expect("unable to decode secret, is your secret in base 64 encoding");
        // hmac-sha256 it
        let mut hmac = crypto::hmac::Hmac::new(crypto::sha2::Sha256::new(), &decoded_secret);
        hmac.input(prehash.as_bytes());
        let hmac_result = hmac.result();
        let hmac_code = hmac_result.code();
        let base64_encoding = base64::encode(hmac_code);
        // return base64 encoded hmac result
        base64_encoding
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>, Error> {
        let accounts = self.get_and_deserialize("/accounts").await?;
        Ok(accounts)
    }

    async fn post_and_deserialize(&self, path: &str) -> Result<String, reqwest::Error> {
        let headers = self.access_headers(path, None, "POST");
        let res = self
            .reqwest_client
            .post(format!("{}{}", COINBASE_API_URL, path))
            .headers(headers)
            .send()
            .await?;
        let status = res.status();
        let text = res.text().await?;
        // return Err(Error::new(ErrorKind::Status(status)));
        println!("{}", status);
        println!("{}", text);
        Ok(text)
    }

    pub async fn place_order(&self, body: Order) -> Result<String, reqwest::Error> {
        Ok(self.post_and_deserialize("./orders").await?)
    }
}

enum Order {
    Limit {
        size: f64,
        price: f64,
        side: OrderSide,
    },
    Market {
        size_or_funds: SizeOrFunds,
        price: f64,
        side: OrderSide,
    },
}

impl Order {
    pub fn limit(size: f64, price: f64, side: OrderSide) -> Self {
        Self::Limit { size, price, side }
    }

    pub fn market(size_or_funds: SizeOrFunds, price: f64, side: OrderSide) -> Self {
        Self::Market {
            size_or_funds,
            price,
            side,
        }
    }
}

enum OrderSide {
    Buy,
    Sell,
}

enum SizeOrFunds {
    Size(f64),
    Funds(f64),
}

#[cfg(test)]
mod tests {

    use super::*;
    use dotenv;
    use std::env;

    fn create_client() -> PrivateClient {
        dotenv::from_filename(".env").expect("error reading .env file");
        let secret = env::var("SECRET").expect("Cant find api secret");
        let passphrase = env::var("PASSPHRASE").expect("Cant find api passphrase");
        let key = env::var("KEY").expect("Cant find api key");
        PrivateClient::new(secret, passphrase, key)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_accounts() {
        let client = create_client();
        let future = client.get_accounts();
        let _json = futures::executor::block_on(future).unwrap();
    }
}
