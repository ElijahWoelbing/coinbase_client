use super::{deserialize_f64, deserialize_response, COINBASE_API_URL};
use crate::error::Error;
use base64;
use core::f64;
use crypto::{self, mac::Mac};
use reqwest;
use serde::{self, Deserialize};
use std::time::{SystemTime, SystemTimeError};

/// `PrivateClient` requires authentication and provide access to placing orders and other account information
pub struct PrivateClient {
    reqwest_client: reqwest::Client,
    secret: String,
    passphrase: String,
    key: String,
    url: &'static str,
}

impl PrivateClient {
    /// Creates a new `PrivateClient`
    pub fn new(secret: String, passphrase: String, key: String) -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            secret, // shared secret
            key,
            passphrase,
            url: COINBASE_API_URL,
        }
    }

    /// Creates a new `PrivateClient` for testing API connectivity and web trading
    pub fn new_sandbox(secret: String, passphrase: String, key: String) -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            secret, // shared secret
            key,
            passphrase,
            url: "https://api-public.sandbox.pro.coinbase.com",
        }
    }

    async fn get<T>(&self, path: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let headers = self.access_headers(path, None, "GET");
        let response = self
            .reqwest_client
            .get(format!("{}{}", self.url, path))
            .headers(headers)
            .send()
            .await?;

        deserialize_response::<T>(response).await
    }

    async fn post<T, K>(&self, path: &str, body: K) -> Result<T, Error>
    where
        K: serde::Serialize,            // body must serialize
        T: serde::de::DeserializeOwned, // response must deserialize
    {
        let headers = self.access_headers(path, Some(&serde_json::to_string(&body)?), "POST");
        let url = format!("{}{}", self.url, path);
        let response = self
            .reqwest_client
            .post(url)
            .headers(headers)
            .json::<K>(&body)
            .send()
            .await?;
        deserialize_response::<T>(response).await
    }

    async fn delete<T>(&self, path: &str) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let headers = self.access_headers(path, None, "DELETE");
        let response = self
            .reqwest_client
            .delete(format!("{}{}", self.url, path))
            .headers(headers)
            .send()
            .await?;
        deserialize_response::<T>(response).await
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

    /// gets a list of trading accounts from the profile of the API key.
    pub async fn get_accounts(&self) -> Result<Vec<Account>, Error> {
        let accounts = self.get("/accounts").await?;
        Ok(accounts)
    }

    /// you can place three types of orders: limit, market and stop [Overview of order types and settings](https://help.coinbase.com/en/pro/trading-and-funding/orders/overview-of-order-types-and-settings-stop-limit-market)
    pub async fn place_order(&self, order: Order) -> Result<String, Error> {
        #[derive(serde::Deserialize, Debug)]
        pub struct OrderID {
            pub id: String,
        }
        Ok(self.post::<OrderID, _>("/orders", order).await?.id)
    }

    /// cancel order specified by order ID
    pub async fn cancel_order(&self, order_id: &str) -> Result<String, Error> {
        Ok(self.delete(&format!("/orders/{}", order_id)).await?)
    }

    // IMPORTANT not tested as OID is not fully supported yet
    /// cancel order specified by order OID
    pub async fn cancel_order_by_oid(&self, oid: &str) -> Result<String, Error> {
        Ok(self.delete(&format!("/orders/client:{}", oid)).await?)
    }

    /// cancel all orders
    pub async fn cancel_orders(&self) -> Result<Vec<String>, Error> {
        Ok(self.delete("/orders").await?)
    }

    /// get open orders from the profile that the API key belongs
    pub async fn get_orders(&self) -> Result<Vec<OrderInfo>, Error> {
        Ok(self.get("/orders").await?)
    }

    /// get open order from the profile that the API key belongs
    pub async fn get_order(&self, order_id: &str) -> Result<OrderInfo, Error> {
        Ok(self.get(&format!("/orders/{}", order_id)).await?)
    }
    // IMPORTANT not tested as OID is not fully supported yet
    /// gets order specified by order OID
    pub async fn get_order_by_oid(&self, oid: &str) -> Result<OrderInfo, Error> {
        Ok(self.get(&format!("/orders/client:{}", oid)).await?)
    }

    /// get recent fills by specified order_id of the API key's profile
    pub async fn get_fills_by_order_id(&self, order_id: &str) -> Result<Vec<Fill>, Error> {
        Ok(self.get(&format!("/fills?order_id={}", order_id)).await?)
    }

    /// get recent fills by specified product_id of the API key's profile
    pub async fn get_fills_by_product_id(&self, product_id: &str) -> Result<Vec<Fill>, Error> {
        Ok(self
            .get(&format!("/fills?product_id={}", product_id))
            .await?)
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub currency: String,
    #[serde(deserialize_with = "deserialize_f64")]
    pub balance: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub available: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    pub hold: f64,
    pub profile_id: String,
    pub trading_enabled: bool,
}

/// A `OrderBuilder` should be used to create a `Order` with  custom configuration.
#[derive(serde::Serialize)]
pub struct Order {
    r#type: String,
    size: Option<f64>,
    price: Option<f64>,
    side: OrderSide,
    client_oid: Option<String>,
    self_trade_prevention: Option<SelfTradePrevention>,
    time_in_force: Option<TimeInForce>,
    cancel_after: Option<CancelAfter>,
    post_only: Option<bool>,
    funds: Option<f64>,
    product_id: String,
    stp: Option<String>,
    stop: Option<OrderStop>,
    stop_price: Option<f64>,
}

/// A `OrderBuilder` can be used to create a `Order` with custom configuration.
/// <br>
/// Confiuguration parameters details can be found [here](https://docs.pro.coinbase.com/#orders)
pub struct OrderBuilder {
    r#type: String,
    size: Option<f64>,
    price: Option<f64>,
    side: OrderSide,
    client_oid: Option<String>,
    self_trade_prevention: Option<SelfTradePrevention>,
    time_in_force: Option<TimeInForce>,
    cancel_after: Option<CancelAfter>,
    post_only: Option<bool>,
    funds: Option<f64>,
    product_id: String,
    stp: Option<String>,
    stop: Option<OrderStop>,
    stop_price: Option<f64>,
}

impl OrderBuilder {
    //! returns a `OrderBuilder` with requiered market-order parameters.
    pub fn market(
        side: OrderSide,
        product_id: &str,
        size_or_funds: SizeOrFunds,
    ) -> impl SharedOptions {
        Self {
            r#type: "market".to_string(),
            size: match size_or_funds {
                SizeOrFunds::Size(n) => Some(n),
                _ => None,
            },
            price: None,
            side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: match size_or_funds {
                SizeOrFunds::Funds(n) => Some(n),
                _ => None,
            },
            product_id: product_id.to_string(),
            stp: None,
            stop: None,
            stop_price: None,
        }
    }

    /// returns a `OrderBuilder` with requiered limit-order parameters.
    pub fn limit(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
    ) -> impl LimitOptions + SharedOptions {
        Self {
            r#type: "limit".to_string(),
            size: Some(size),
            price: Some(price),
            side: side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: None,
            product_id: product_id.to_string(),
            stp: None,
            stop: None,
            stop_price: None,
        }
    }

    /// returns a `OrderBuilder` with requiered stop-order parameters.
    pub fn stop(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
        stop_price: f64,
        stop: OrderStop,
    ) -> impl SharedOptions {
        Self {
            r#type: "limit".to_string(),
            size: Some(size),
            price: Some(price),
            side: side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: None,
            product_id: product_id.to_string(),
            stp: None,
            stop: Some(stop),
            stop_price: Some(stop_price),
        }
    }
}

/// 'SharedOptions' options can be used with market, limit and stop order types
pub trait SharedOptions {
    fn self_trade_prevention(self, self_trade_prevention: SelfTradePrevention) -> Self;
    fn client_oid(self, client_oid: String) -> Self;
    fn build(self) -> Order;
}

impl SharedOptions for OrderBuilder {
    /// Sets the Orders self-trade behavior
    fn self_trade_prevention(mut self, self_trade_prevention: SelfTradePrevention) -> Self {
        self.self_trade_prevention = Some(self_trade_prevention);
        self
    }

    /// Sets the Order ID to identify your order
    fn client_oid(mut self, client_oid: String) -> Self {
        self.client_oid = Some(client_oid);
        self
    }

    /// Builds `Order`
    fn build(self) -> Order {
        Order {
            r#type: self.r#type,
            size: self.size,
            price: self.price,
            side: self.side,
            client_oid: self.client_oid,
            self_trade_prevention: self.self_trade_prevention,
            time_in_force: self.time_in_force,
            cancel_after: self.cancel_after,
            post_only: self.post_only,
            funds: self.funds,
            product_id: self.product_id,
            stp: self.stp,
            stop: self.stop,
            stop_price: self.stop_price,
        }
    }
}

/// Builder options for Limit Orders
pub trait LimitOptions {
    fn time_in_force(self, time_in_force: TimeInForce) -> Self;
}

impl LimitOptions for OrderBuilder {
    /// This option provides guarantees about the lifetime of an Order
    fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        match time_in_force {
            TimeInForce::GoodTillTime {
                cancel_after,
                post_only,
            } => {
                self.cancel_after = Some(cancel_after);
                self.post_only = Some(post_only);
            }
            TimeInForce::GoodTillCancel { post_only } => self.post_only = Some(post_only),
            _ => {}
        }
        self.time_in_force = Some(time_in_force);
        self
    }
}

#[derive(Debug, Deserialize)]
pub struct OrderInfo {
    id: String,
    #[serde(deserialize_with = "deserialize_f64")]
    price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    size: f64,
    product_id: String,
    side: String,
    stp: Option<String>,
    r#type: String,
    time_in_force: String,
    post_only: bool,
    created_at: String,
    #[serde(deserialize_with = "deserialize_f64")]
    fill_fees: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    filled_size: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    executed_value: f64,
    status: String,
    settled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Fill {
    trade_id: u64,
    product_id: String,
    #[serde(deserialize_with = "deserialize_f64")]
    price: f64,
    #[serde(deserialize_with = "deserialize_f64")]
    size: f64,
    order_id: String,
    created_at: String,
    liquidity: String,
    #[serde(deserialize_with = "deserialize_f64")]
    fee: f64,
    settled: bool,
    side: String,
}

#[derive(Clone, Copy, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}
#[derive(Clone, Copy, Debug)]
pub enum OrderStop {
    Loss,  // Triggers when the last trade price changes to a value at or below the stop_price.
    Entry, // Triggers when the last trade price changes to a value at or above the stop_price.
}

#[derive(Clone, Copy, Debug)]
pub enum SizeOrFunds {
    Size(f64),
    Funds(f64),
}

#[derive(Clone, Copy, Debug)]
pub enum TimeInForce {
    GoodTillCancel {
        post_only: bool,
    },
    GoodTillTime {
        cancel_after: CancelAfter,
        post_only: bool,
    },
    ImmediateOrCancel,
    FillOrKill,
}

#[derive(Clone, Copy, Debug)]
pub enum CancelAfter {
    Minute,
    Hour,
    Day,
}

#[derive(Clone, Copy, Debug)]
pub enum SelfTradePrevention {
    DecreaseCancel,
    CancelOldest,
    CancelNewest,
    CancelBoth,
}

impl serde::Serialize for SelfTradePrevention {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            SelfTradePrevention::CancelBoth => serializer.serialize_str("cb"),
            SelfTradePrevention::DecreaseCancel => serializer.serialize_str("dc"),
            SelfTradePrevention::CancelOldest => serializer.serialize_str("co"),
            SelfTradePrevention::CancelNewest => serializer.serialize_str("cn"),
        }
    }
}

impl serde::Serialize for OrderStop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Loss => serializer.serialize_str("loss"),
            Self::Entry => serializer.serialize_str("entry"),
        }
    }
}

impl serde::Serialize for TimeInForce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::GoodTillCancel { post_only: _ } => serializer.serialize_str("GTC"),
            Self::GoodTillTime {
                cancel_after: _,
                post_only: _,
            } => serializer.serialize_str("GTT"),
            Self::ImmediateOrCancel => serializer.serialize_str("IOC"),
            Self::FillOrKill => serializer.serialize_str("FOK"),
        }
    }
}

impl serde::Serialize for CancelAfter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Minute => serializer.serialize_str("min"),
            Self::Hour => serializer.serialize_str("hour"),
            Self::Day => serializer.serialize_str("day"),
        }
    }
}

impl serde::Serialize for OrderSide {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            OrderSide::Buy => serializer.serialize_str("buy"),
            OrderSide::Sell => serializer.serialize_str("sell"),
        }
    }
}

impl serde::Serialize for SizeOrFunds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            Self::Size(size) => serializer.serialize_f64(size),
            Self::Funds(funds) => serializer.serialize_f64(funds),
        }
    }
}
