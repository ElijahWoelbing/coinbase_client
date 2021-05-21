use super::error::{Error, ErrorKind};
use super::json::*;
use base64;
use chrono::format::format;
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

    async fn post_and_deserialize(
        &self,
        path: &str,
        body: OrderParams,
    ) -> Result<String, reqwest::Error> {
        let body_json = serde_json::to_string(&body).unwrap();
        let headers = self.access_headers(path, None, "POST");
        let res = self
            .reqwest_client
            .post(format!("{}{}", COINBASE_API_URL, path))
            .body(body_json)
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

    pub async fn place_order(&self, body: OrderParams) -> Result<String, reqwest::Error> {
        Ok(self.post_and_deserialize("./orders", body).await?)
    }
}

pub struct OrderParamsBuilder {
    size: Option<String>,
    price: String,
    product_id: String,
    side: OrderSide,
    order_type: OrderType,
    client_oid: Option<Uuid>,
    stp: SelfTradePervention,
    stop: Option<OrderStop>,
    stop_price: Option<String>,
    time_in_force: Option<TimeInForce>,
    cancel_after: Option<OrderTime>,
    post_only: Option<bool>,
    funds: Option<String>,
}

impl OrderParamsBuilder {
    pub fn new(price: String, side: OrderSide, product_id: String, order_type: OrderType) -> Self {
        Self {
            size: match order_type {
                OrderType::Market { market_type } => match market_type {
                    MarketType::Size { size } => Some(size),
                    MarketType::SizeAndFunds { size, funds } => Some(size),
                    _ => None,
                },
                OrderType::Limit {
                    size,
                    time_in_force,
                } => Some(size),
            },
            price,
            side,
            product_id,
            order_type,
            client_oid: None,
            stp: SelfTradePervention::DecreaseAndCancel,
            stop: None,
            stop_price: None,
            time_in_force: match order_type {
                OrderType::Limit {
                    size,
                    time_in_force,
                } => Some(time_in_force),
                _ => None,
            },
            cancel_after: match order_type {
                OrderType::Limit {
                    size,
                    time_in_force,
                } => match time_in_force {
                    TimeInForce::GoodTillTime { time, post_only } => Some(time),
                    _ => None,
                },
                _ => None,
            },
            post_only: match order_type {
                OrderType::Limit {
                    size,
                    time_in_force,
                } => match time_in_force {
                    TimeInForce::GoodTillTime { time, post_only } => Some(post_only),
                    TimeInForce::GoodTillCanceled { post_only } => Some(post_only),
                    _ => None,
                },
                _ => None,
            },
            funds: match order_type {
                OrderType::Market { market_type } => match market_type {
                    MarketType::Funds { funds } => Some(funds),
                    MarketType::SizeAndFunds { size, funds } => Some(funds),
                    _ => None,
                },
                _ => None,
            },
        }
    }

    pub fn set_client_oid(mut self, uuid: Uuid) -> Self {
        self.client_oid = Some(uuid);
        self
    }

    pub fn set_stp(mut self, stp: SelfTradePervention) -> Self {
        self.stp = stp;
        self
    }

    pub fn set_stop(mut self, stop: OrderStop, stop_price: String) -> Self {
        self.stop_price = Some(stop_price);
        self.stop = Some(stop);
        self
    }

    pub fn build(self) -> OrderParams {
        OrderParams {
            size: self.size,
            price: self.price,
            side: match self.side {
                OrderSide::Buy => "buy".to_string(),
                OrderSide::Sell => "sell".to_string(),
            },
            product_id: self.product_id,
            r#type: match self.order_type {
                OrderType::Limit {
                    size: _,
                    time_in_force: _,
                } => "limit".to_string(),
                OrderType::Market { market_type: _ } => "market".to_string(),
            },
            client_oid: match self.client_oid {
                Some(uuid) => Some(uuid.to_string()),
                None => None,
            },
            stp: match self.stp {
                SelfTradePervention::DecreaseAndCancel => "dc".to_string(),
                SelfTradePervention::CancelOldest => "co".to_string(),
                SelfTradePervention::CancelNewest => "cn".to_string(),
                SelfTradePervention::CancelBoth => "cb".to_string(),
            },
            stop: match self.stop {
                Some(stop) => match stop {
                    OrderStop::Entry => Some("entry".to_string()),
                    OrderStop::Loss => Some("loss".to_string()),
                },
                None => None,
            },
            stop_price: self.stop_price,
            time_in_force: match self.time_in_force {
                Some(TimeInForce::GoodTillCanceled { post_only: _ }) => Some("GTC".to_string()),
                Some(TimeInForce::GoodTillTime {
                    time: _,
                    post_only: _,
                }) => Some("GTT".to_string()),
                Some(TimeInForce::ImmediateOrCancel) => Some("IOC".to_string()),
                Some(TimeInForce::FillOrKill) => Some("FOK".to_string()),
                None => None,
            },
            cancel_after: match self.cancel_after {
                Some(order_time) => match order_time {
                    OrderTime::OneMinute => Some("min".to_string()),
                    OrderTime::OneHour => Some("hour".to_string()),
                    OrderTime::OneDay => Some("day".to_string()),
                },
                None => None,
            },
            post_only: self.post_only,
            funds: self.funds,
        }
    }
}

#[derive(Serialize)]
struct OrderParams {
    size: Option<String>,
    price: String,
    product_id: String,
    side: String,
    r#type: String,
    client_oid: Option<String>,
    stp: String,
    stop: Option<String>,
    stop_price: Option<String>,
    time_in_force: Option<String>,
    cancel_after: Option<String>,
    post_only: Option<bool>,
    funds: Option<String>,
}

pub enum OrderType {
    Market {
        market_type: MarketType,
    },
    Limit {
        size: String,
        time_in_force: TimeInForce,
    },
}

pub enum MarketType {
    Size { size: String },
    Funds { funds: String },
    SizeAndFunds { size: String, funds: String },
}

#[derive(Serialize)]
pub enum OrderTime {
    OneMinute,
    OneHour,
    OneDay,
}
#[derive(Serialize)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Serialize)]
pub enum OrderStop {
    Loss,
    Entry,
}

#[derive(Serialize)]
pub enum TimeInForce {
    GoodTillCanceled { post_only: bool },
    GoodTillTime { time: OrderTime, post_only: bool },
    ImmediateOrCancel,
    FillOrKill,
}

#[derive(Serialize)]
enum SelfTradePervention {
    DecreaseAndCancel,
    CancelOldest,
    CancelNewest,
    CancelBoth,
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_get_place_order() {
        let client = create_client();
        let future = client.place_order();
        let _json = futures::executor::block_on(future).unwrap();
    }
}
