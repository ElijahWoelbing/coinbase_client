use crate::configure_pagination;
use crate::{
    deserialize_option_to_date, deserialize_response, deserialize_to_date, Json, COINBASE_API_URL,
    COINBASE_SANDBOX_API_URL,
};

use super::Order;
use super::Report;

use crate::error::{Error, ErrorKind, ErrorMessage, StatusError};
use base64;
use chrono::{DateTime, Utc};
use core::f64;
use crypto::{self, mac::Mac};
use reqwest;
use serde::{self, Deserialize};
use std::str;
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
    async fn get_paginated<T>(
        &self,
        path: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let pagination_params = configure_pagination(before, after, limit);
        self.get(&format!("{}{}", path, pagination_params)).await
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

    async fn post_and_deserialize<T, K>(&self, path: &str, body: Option<K>) -> Result<T, Error>
    where
        K: serde::Serialize,
        T: serde::de::DeserializeOwned,
    {
        deserialize_response::<T>(self.post(path, body).await?).await
    }

    async fn post<K>(&self, path: &str, body: Option<K>) -> Result<reqwest::Response, Error>
    where
        K: serde::Serialize,
    {
        let request_builder = self.reqwest_client.post(format!("{}{}", self.url, path));
        Ok(if let Some(n) = body {
            request_builder
                .headers(self.access_headers(path, Some(&serde_json::to_string(&n)?), "POST"))
                .json::<K>(&n)
                .send()
        } else {
            request_builder
                .headers(self.access_headers(path, None, "POST"))
                .send()
        }
        .await?)
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

    fn sign_message(
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

    /// Creates a new `PrivateClient`
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// ~~~~
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
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// ~~~~
    pub fn new_sandbox(secret: String, passphrase: String, key: String) -> Self {
        Self {
            reqwest_client: reqwest::Client::new(),
            secret,
            key,
            passphrase,
            url: COINBASE_SANDBOX_API_URL,
        }
    }

    /// Gets a list of trading accounts from the profile of the API key.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#account)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let accounts = client.get_accounts().await.unwrap();
    /// ~~~~
    pub async fn get_accounts(&self) -> Result<Vec<Account>, Error> {
        let accounts = self.get("/accounts").await?;
        Ok(accounts)
    }

    /// Get trading account by account ID
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#account)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let account = client.get_account("1f6a7175-a89c-494f-986d-af9987e6dd69")
    /// .await
    /// .unwrap();
    /// ~~~~
    pub async fn get_account(&self, account_id: &str) -> Result<Account, Error> {
        let account = self.get(&format!("/accounts/{}", account_id)).await?;
        Ok(account)
    }

    /// Get account activity of the API key's profile.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-account-history)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    ///  let history = client
    /// .get_account_history(
    ///     "680f85f4-1a99-4108-93ce-a9066f9de246",
    ///     Some("297946691"),
    ///     Some("296147671"),
    ///     Some(100),
    /// )
    /// .await
    /// .unwrap();
    /// ~~~~
    pub async fn get_account_history(
        &self,
        account_id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<AccountHistory>, Error> {
        let account = self
            .get_paginated(
                &format!("/accounts/{}/ledger?", account_id),
                before,
                after,
                limit,
            )
            .await?;
        Ok(account)
    }

    /// Get holds of an account that belong to the same profile as the API key.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-holds)\
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let _holds = client
    /// .get_account_holds(
    ///     "680f85f4-1a99-4108-93ce-a9066f9de246",
    ///     None,
    ///     None,
    ///     Some(100),
    /// )
    /// .await
    /// .unwrap();
    /// ~~~~
    pub async fn get_account_holds(
        &self,
        account_id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<Hold>, Error> {
        let account = self
            .get_paginated(
                &format!("/accounts/{}/holds?", account_id),
                before,
                after,
                limit,
            )
            .await?;
        Ok(account)
    }

    /// You can place three types of orders: limit, market and stop
    /// <br>
    /// [Overview of order types and settings](https://help.coinbase.com/en/pro/trading-and-funding/orders/overview-of-order-types-and-settings-stop-limit-market)
    /// <br>
    /// Create order order useing [`OrderBuilder`](https://docs.rs/coinbase-client/1.0.0-alpha/coinbase_client/private_client/struct.OrderBuilder.html)
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#place-a-new-order)
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let order = OrderBuilder::market(OrderSide::Buy, "BTC-USD", SizeOrFunds::Funds(10.00))
    /// .build();
    /// let res = client.place_order(order).await.unwrap();
    /// ~~~~
    pub async fn place_order(&self, order: Order) -> Result<String, Error> {
        #[derive(Deserialize, Debug)]
        pub struct OrderID {
            pub id: String,
        }
        Ok(self
            .post_and_deserialize::<OrderID, _>("/orders", Some(order))
            .await?
            .id)
    }

    /// Cancel order specified by order ID
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#cancel-an-order)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", 33000.0, 1.0)
    /// .build();
    /// let order_to_cancel_id = client.place_order(order).await.unwrap();
    /// let canceled_order_id = client.cancel_order(&order_to_cancel_id)
    /// .await.unwrap();
    /// ~~~~
    pub async fn cancel_order(&self, order_id: &str) -> Result<String, Error> {
        Ok(self.delete(&format!("/orders/{}", order_id)).await?)
    }

    /// Cancel order specified by order OID
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#cancel-an-order)
    pub async fn cancel_order_by_oid(&self, oid: &str) -> Result<String, Error> {
        Ok(self.delete(&format!("/orders/client:{}", oid)).await?)
    }

    /// Cancel all orders
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#cancel-an-order)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let canceled_orders_ids = client.cancel_orders().await.unwrap();
    /// ~~~~
    pub async fn cancel_orders(&self) -> Result<Vec<String>, Error> {
        Ok(self.delete("/orders").await?)
    }

    /// Get open orders from the profile that the API key belongs
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#list-orders)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let orders = client
    ///     .get_orders(
    ///         Some(OrderStatus::OpenActivePending),
    ///         Some("2021-06-19T20:24:20.467086Z"),
    ///         None,
    ///         None,
    ///     )
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_orders(
        &self,
        order_status: Option<OrderStatus>,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<OrderInfo>, Error> {
        let path = match order_status {
            Some(n) => {
                let params = match n {
                    OrderStatus::Open => "status=open",
                    OrderStatus::Active => "status=active",
                    OrderStatus::Pending => "status=pending",
                    OrderStatus::OpenActive => "status=open&status=active",
                    OrderStatus::OpenPending => "status=open&status=pending",
                    OrderStatus::ActivePending => "status=active&status=pending",
                    OrderStatus::OpenActivePending => "status=open&status=active&status=pending",
                };
                format!("/orders?{}&", params)
            }
            None => String::from("/orders?"),
        };

        Ok(self.get_paginated(&path, before, after, limit).await?)
    }

    /// Get open order from the profile that the API key belongs
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-an-order)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", 36000.0, 1.0)
    /// .build();
    /// let order_id = client.place_order(order).await.unwrap();
    /// let order = client.get_order(&order_id).await.unwrap();
    /// ~~~~
    pub async fn get_order(&self, order_id: &str) -> Result<OrderInfo, Error> {
        Ok(self.get(&format!("/orders/{}", order_id)).await?)
    }

    /// Gets order specified by order OID
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-an-order)
    pub async fn get_order_by_oid(&self, oid: &str) -> Result<OrderInfo, Error> {
        Ok(self.get(&format!("/orders/client:{}", oid)).await?)
    }

    /// Get recent fills by specified order_id of the API key's profile
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#fills)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let fills = client
    ///     .get_fill_by_order_id("4f2756cf-dcb5-492b-83e5-5f2141892758", None, None, None)
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_fill_by_order_id(
        &self,
        order_id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<Fill>, Error> {
        Ok(self
            .get_paginated(
                &format!("/fills?order_id={}&", order_id),
                before,
                after,
                limit,
            )
            .await?)
    }

    /// Get recent fills by specified product_id of the API key's profile
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#fills)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let fills = client
    ///     .get_fills_by_product_id(&product_id, None, Some("29786034"), None)
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_fills_by_product_id(
        &self,
        product_id: &str,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Vec<Fill>, Error> {
        Ok(self
            .get_paginated(
                &format!("/fills?product_id={}&", product_id),
                before,
                after,
                limit,
            )
            .await?)
    }

    /// Get information on your payment method transfer limits, as well as buy/sell limits per currency
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#limits)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let limits = client.get_limits().await.unwrap();
    /// ~~~~
    pub async fn get_limits(&self) -> Result<Json, Error> {
        Ok(self.get(&format!("/users/self/exchange-limits")).await?)
    }

    /// Get deposits from the profile of the API key, in descending order by created time
    /// <br>
    /// **optional parameters**
    /// <br>
    /// *profile_id*: limit list of deposits to this profile_id. By default, it retrieves deposits using default profile
    /// <br>
    /// *before*: if before is set, then it returns deposits created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// *after*: if after is set, then it returns deposits created before the after timestamp, sorted by newest
    /// <br>
    /// *limit*: truncate list to this many deposits, capped at 100. Default is 100.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#list-deposits)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let deposits = client
    ///     .get_deposits(
    ///         Some("b7482eaa-3eea-4065-9d81-1484257c5f92"),
    ///         None,
    ///         None,
    ///         None,
    ///     )
    ///     .await.unwrap();
    /// ~~~~
    pub async fn get_deposits(
        &self,
        profile_id: Option<&str>,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Json, Error> {
        let path = match profile_id {
            Some(n) => format!("/transfers?type=deposit&profile_id={}&", n),
            None => String::from("/transfers?type=deposit&"),
        };
        Ok(self.get_paginated(&path, before, after, limit).await?)
    }
    /// Get internal deposits from the profile of the API key, in descending order by created time
    /// <br>
    /// **optional parameters**
    /// <br>
    /// *profile_id*: limit list of internal deposits to this profile_id. By default, it retrieves internal deposits using default profile
    /// <br>
    /// *before*: if before is set, then it returns internal deposits created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// *after*: if after is set, then it returns internal deposits created before the after timestamp, sorted by newest
    /// <br>
    /// *limit*: truncate list to this many internal deposits, capped at 100. Default is 100.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#list-deposits)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let deposits = client
    /// .get_internal_deposits(
    ///     Some("e1d7731f-b7e2-4285-b711-eeec76fc2aff"),
    ///     None,
    ///     None,
    ///     None,
    /// )
    /// .await.unwrap();
    /// ~~~~
    pub async fn get_internal_deposits(
        &self,
        profile_id: Option<&str>,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Json, Error> {
        let path = match profile_id {
            Some(n) => format!("/transfers?type=internal_deposit&profile_id={}&", n),
            None => String::from("/transfers?type=internal_deposit&"),
        };
        Ok(self.get_paginated(&path, before, after, limit).await?)
    }

    /// Get information on a single deposit
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#single-deposit)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let deposit = client
    /// .get_deposit("80259339-7bf9-498f-8200-ddbd32a1c545")
    /// .await;
    /// ~~~~
    pub async fn get_deposit(&self, transfer_id: &str) -> Result<Json, Error> {
        Ok(self.get(&format!("/transfers/{}", transfer_id)).await?)
    }

    /// Get your payment methods
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#payment-methods)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let payment_methods = client.get_payment_methods().await.unwrap();
    /// ~~~~
    pub async fn get_payment_methods(&self) -> Result<Json, Error> {
        Ok(self.get("/payment-methods").await?)
    }

    /// Deposit funds from a payment method
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#payment-method)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let res = client
    /// .deposit_funds(10.00, "USD", "1b4b4fbc-8921-5e7c-b362-a1c589a2cf20")
    /// .await
    /// .unwrap();
    /// ~~~~
    pub async fn deposit_funds(
        &self,
        amount: f64,
        currency: &str,
        payment_method_id: &str,
    ) -> Result<DepositInfo, Error> {
        Ok(self
            .post_and_deserialize(
                "/deposits/payment-method",
                Some(serde_json::json!({
                        "amount": amount,
                        "currency": currency,
                        "payment_method_id": payment_method_id
                })),
            )
            .await?)
    }

    /// Deposit funds from a coinbase account
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#coinbase)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let res = client
    ///     .deposit_funds_from_coinbase(10.00, "BTC", "95671473-4dda-5264-a654-fc6923e8a334")
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn deposit_funds_from_coinbase(
        &self,
        amount: f64,
        currency: &str,
        coinbase_account_id: &str,
    ) -> Result<DepositInfo, Error> {
        Ok(self
            .post_and_deserialize(
                "/deposits/coinbase-account",
                Some(serde_json::json!({
                        "amount": amount,
                        "currency": currency,
                        "coinbase_account_id": coinbase_account_id
                })),
            )
            .await?)
    }

    /// Get a list of your coinbase accounts
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#coinbase-accounts)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let accounts = client.get_coinbase_accounts().await.unwrap();
    /// ~~~~
    pub async fn get_coinbase_accounts(&self) -> Result<Json, Error> {
        Ok(self.get("/coinbase-accounts").await?)
    }

    /// Generate an address for crypto deposits
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#generate-a-crypto-deposit-address)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let address = client
    ///     .generate_crypto_deposit_address("95671473-4dda-5264-a654-fc6923e8a334")
    ///     .await
    ///     .unwrap();    
    /// ~~~~
    pub async fn generate_crypto_deposit_address(
        &self,
        coinbase_account_id: &str,
    ) -> Result<Json, Error> {
        Ok(self
            .post_and_deserialize::<_, Json>(
                &format!("/coinbase-accounts/{}/addresses", coinbase_account_id),
                None,
            )
            .await?)
    }

    /// Get withdrawals from the profile of the API key
    /// <br>
    /// **optional parameters**
    /// <br>
    /// *profile_id*: limit list of withdrawals to this profile_id. By default, it retrieves withdrawals using default profile
    /// <br>
    /// *before*: If before is set, then it returns withdrawals created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// *after*: If after is set, then it returns withdrawals created before the after timestamp, sorted by newest
    /// <br>
    /// *limit*: truncate list to this many withdrawals, capped at 100. Default is 100
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#list-withdrawals)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let withdrawls = client
    ///     .get_withdrawls(
    ///         Some("b7482eaa-3eea-4065-9d81-1484257c5f92"),
    ///         None,
    ///         None,
    ///         None,
    ///     )
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_withdrawls(
        &self,
        profile_id: Option<&str>,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Json, Error> {
        let path = match profile_id {
            Some(n) => format!("/transfers?type=withdraw&profile_id={}&", n),
            None => String::from("/transfers?type=withdraw&"),
        };
        Ok(self.get_paginated(&path, before, after, limit).await?)
    }

    /// Get withdrawals from the profile of the API key
    /// <br>
    /// **optional parameters**
    /// <br>
    /// *profile_id*: limit list of internal withdrawals to this profile_id. By default, it retrieves internal withdrawals using default profile
    /// <br>
    /// <br>
    /// *before*: If before is set, then it returns internal withdrawals created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// <br>
    /// *after*: If after is set, then it returns internal withdrawals created before the after timestamp, sorted by newest
    /// <br>
    /// <br>
    /// *limit*: truncate list to this many internal withdrawals, capped at 100. Default is 100
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#list-withdrawals)
    /// <br>
    /// This request is [paginated](https://docs.pro.coinbase.com/#pagination)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let withdrawls = client
    ///     .get_internal_withdrawls(
    ///         Some("b7482eaa-3eea-4065-9d81-1484257c5f92"),
    ///         None,
    ///         None,
    ///         None,
    ///     )
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_internal_withdrawls(
        &self,
        profile_id: Option<&str>,
        before: Option<&str>,
        after: Option<&str>,
        limit: Option<u16>,
    ) -> Result<Json, Error> {
        let path = match profile_id {
            Some(n) => format!("/transfers?type=internal_withdraw&profile_id={}&", n),
            None => String::from("/transfers?type=internal_withdraw&"),
        };
        Ok(self.get_paginated(&path, before, after, limit).await?)
    }

    /// Get information on a single withdrawal
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#single-withdrawal)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let withdrawl = client
    ///     .get_withdrawl("0e94a87f-9d50-4ead-86ac-7898830c5edf")
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_withdrawl(&self, transfer_id: &str) -> Result<Json, Error> {
        Ok(self.get(&format!("/transfers/{}", transfer_id)).await?)
    }

    /// Withdraw funds to a payment method
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#payment-method55)
    pub async fn withdraw_funds(
        &self,
        amount: f64,
        currency: &str,
        payment_method_id: &str,
    ) -> Result<WithdrawInfo, Error> {
        Ok(self
            .post_and_deserialize(
                "/withdrawals/payment-method",
                Some(serde_json::json!({
                        "amount": amount,
                        "currency": currency,
                        "payment_method_id": payment_method_id
                })),
            )
            .await?)
    }

    /// Withdraw funds to a coinbase account
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#coinbase56)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let res = client
    ///     .withdraw_to_coinbase(1.0, "ADA", "91bdfea7-f2sd-5waa-bb0d-5b93c9f09ffc")
    ///     .await
    ///     .unwrap();    
    /// ~~~~
    pub async fn withdraw_to_coinbase(
        &self,
        amount: f64,
        currency: &str,
        coinbase_account_id: &str,
    ) -> Result<WithdrawInfo, Error> {
        Ok(self
            .post_and_deserialize(
                "/withdrawals/coinbase-account",
                Some(serde_json::json!({
                        "amount": amount,
                        "currency": currency,
                        "coinbase_account_id": coinbase_account_id
                })),
            )
            .await?)
    }

    /// Withdraw funds to a crypto address.
    /// <br>
    /// **parameters**
    /// <br>
    /// amount: The amount to withdraw
    /// <br>
    /// currency: The type of currency
    /// <br>
    /// crypto_address: A crypto address of the recipient
    /// <br>
    /// destination_tag: A destination tag for currencies that support one
    /// <br>
    /// no_destination_tag:	A boolean flag to opt out of using a destination tag for currencies that support one. This is required when not providing a destination tag.
    /// <br>
    /// add_network_fee_to_total: A boolean flag to add the network fee on top of the amount. If this is blank, it will default to deducting the network fee from the amount.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#crypto)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let res = client.withdraw_to_crypto_address(6.0, "ADA", "addr1qyk0yr3ht9d6hcqwp8q8j38nxs04npyjauzz9wp5jcfr95h64lvegfk57zmzltj3nmpjff6490ayyvjh0g6sne6hm3hspnnscy", None, None, None).await.unwrap();
    /// ~~~~
    pub async fn withdraw_to_crypto_address(
        &self,
        amount: f64,
        currency: &str,
        crypto_address: &str,
        destination_tag: Option<&str>,
        no_destination_tag: Option<bool>,
        add_network_fee_to_total: Option<bool>,
    ) -> Result<Json, Error> {
        Ok(self
            .post_and_deserialize(
                "/withdrawals/crypto",
                Some(serde_json::json!({
                        "amount": amount,
                        "currency": currency,
                        "crypto_address": crypto_address,
                        "destination_tag": destination_tag,
                        "no_destination_tag": no_destination_tag,
                        "add_network_fee_to_total": add_network_fee_to_total
                })),
            )
            .await?)
    }

    /// Get your current maker & taker fee rates, as well as your 30-day trailing volume
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-current-fees)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let fees = client.get_fees().await.unwrap();
    /// ~~~~
    pub async fn get_fees(&self) -> Result<Fees, Error> {
        Ok(self.get("/fees").await?)
    }

    /// Get the network fee estimate when sending to the given address
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#fee-estimate)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let fee = client
    ///     .get_fee_estimate("ETH", "0x82289D45Ee8E806C63Ba0DC94a22d4238525d815")
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn get_fee_estimate(
        &self,
        currency: &str,
        crypto_address: &str,
    ) -> Result<f64, Error> {
        #[derive(serde::Deserialize)]
        struct Fee {
            fee: f64,
        }
        let fee = self
            .get::<Fee>(&format!(
                "/withdrawals/fee-estimate?currency={}&crypto_address={}",
                currency, crypto_address
            ))
            .await?;
        Ok(fee.fee)
    }

    /// Convert between stablecoins
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#stablecoin-conversions)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let convertion = client
    ///     .convert_stablecoin("USD", "USDC", 10.00)
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn convert_stablecoin(
        &self,
        from_currency_id: &str,
        to_currency_id: &str,
        amount: f64,
    ) -> Result<StablecoinConversion, Error> {
        Ok(self
            .post_and_deserialize(
                "/conversions",
                Some(serde_json::json!({
                    "from": from_currency_id,
                    "to": to_currency_id,
                    "amount": amount
                })),
            )
            .await?)
    }
    
    /// Reports provide batches of historic information about your profile in various human and machine readable forms    
    /// <br>
    /// Create a `Report` useing [`ReportBuilder`](https://docs.rs/coinbase-client/1.0.0-alpha/coinbase_client/private_client/struct.ReportBuilder.html)
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#create-a-new-report)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let report = Report::account_builder(
    ///     "2014-11-01T00:00:00.000Z",
    ///     "2021-06-11T02:48:15.853Z",
    ///     "1f6a7175-a89c-494f-986d-af9987e6dd69",
    /// )
    /// .email("willstanhope@gmail.com")
    /// .format(Format::CSV)
    /// .build();
    /// let res = client.create_report(report).await.unwrap();
    /// ~~~~
    pub async fn create_report<'a>(&self, report: Report) -> Result<ReportInfo, Error> {
        Ok(self.post_and_deserialize("/reports", Some(report)).await?)
    }

    /// Get report status
    /// <br>
    /// Once a report request has been accepted for processing, the status becomes available
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-report-status)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let report = client
    /// .get_report("d4a3e847-b618-454d-bcb3-e77b0ad61600")
    /// .await
    /// .unwrap();
    /// ~~~~
    pub async fn get_report(&self, report_id: &str) -> Result<ReportInfo, Error> {
        Ok(self.get(&format!("/reports/{}", report_id)).await?)
    }

    /// Get your profiles
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#list-profiles)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let profiles = client.get_profiles().await.unwrap();
    /// ~~~~
    pub async fn get_profiles(&self) -> Result<Vec<Profile>, Error> {
        Ok(self.get("/profiles").await?)
    }

    /// Get a single profile by profile id
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#get-a-profile)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let profile = client
    /// .get_profile("e1d7731f-b7e2-4285-b711-eeec76fc2aff")
    /// .await
    /// .unwrap();
    /// ~~~~
    pub async fn get_profile(&self, profile_id: &str) -> Result<Profile, Error> {
        Ok(self.get(&format!("/profiles/{}", profile_id)).await?)
    }

    /// Transfer funds from API key's profile to another user owned profile
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#create-profile-transfer)
    /// <br>
    /// ~~~~
    /// let client = PrivateClient::new("tGJSu7SuV3/HOR1/9DcFwO1s560BKI51SDEbnwuvTPbw4BbG5lYJLuKUFpD8TPU61R85dxJpGTygKZ5v+6wJdA==", "t9riylyad0r", "4a9f6de8bcdee641a0a207613dfb43ef");
    /// let ok = client
    ///     .create_profile_transfer(
    ///         "e1d7731f-b7e2-4285-b711-eeec76fc2aff",
    ///         "3510ac37-1a99-4c9c-9865-15f1bc5a832e",
    ///         "USD",
    ///         100.00,
    ///     )
    ///     .await
    ///     .unwrap();
    /// ~~~~
    pub async fn create_profile_transfer(
        &self,
        from: &str,
        to: &str,
        currency: &str,
        amount: f64,
    ) -> Result<String, Error> {
        let response = self
            .post(
                "/profiles/transfer",
                Some(serde_json::json!(
                    {
                        "from": from,
                        "to": to,
                        "currency": currency,
                        "amount": amount
                    }
                )),
            )
            .await?;
        let status = response.status();
        if !status.is_success() {
            let error_message = response.json::<ErrorMessage>().await?;
            return Err(Error::new(ErrorKind::Status(StatusError::new(
                status.as_u16(),
                error_message.message,
            ))));
        }
        Ok(response.text().await?)
    }

    /// Get cryptographically signed prices ready to be posted on-chain using Open Oracle smart contracts.
    /// <br>
    /// [API docs](https://docs.pro.coinbase.com/#oracle)
    pub async fn oracle(&self) -> Result<Json, Error> {
        Ok(self.get("/oracle").await?)
    }
}

/// Limit list of orders to these statuses. Passing `OpenActivePending` returns orders of all statuses.
pub enum OrderStatus {
    Open,
    Active,
    Pending,
    OpenActive,
    OpenPending,
    ActivePending,
    OpenActivePending,
}

/// A structure that repersents a Stablecoin Conversion
#[derive(Deserialize, Debug)]
pub struct StablecoinConversion {
    id: String,
    amount: String,
    from_account_id: String,
    to_account_id: String,
    from: String,
    to: String,
}

/// A structure that repersents an Account
#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub currency: String,
    pub balance: String,
    pub available: String,
    pub hold: String,
    pub profile_id: String,
    pub trading_enabled: bool,
}

/// A structure that repersents an Account History
#[derive(Deserialize, Debug)]
pub struct AccountHistory {
    id: String,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
    amount: String,
    balance: String,
    r#type: String,
    details: AccountHistoryDetails,
}

/// A structure that repersents an Account Hold
#[derive(Deserialize, Debug)]
pub struct Hold {
    id: String,
    account_id: String,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_to_date")]
    updated_at: DateTime<Utc>,
    amount: String,
    r#type: String,
    r#ref: String,
}

/// A structure that repersents Account History Details
#[derive(Deserialize, Debug)]
pub struct AccountHistoryDetails {
    order_id: Option<String>,
    trade_id: Option<String>,
    product_id: Option<String>,
}

/// A structure that repersents Deposit Info
#[derive(Deserialize, Debug)]
pub struct DepositInfo {
    id: String,
    amount: String,
    currency: String,
    payout_at: Option<String>,
}

/// A structure that repersents Withdraw Info
#[derive(Deserialize, Debug)]
pub struct WithdrawInfo {
    id: String,
    amount: String,
    currency: String,
}

/// A structure that repersents Order Info
#[derive(Debug, Deserialize)]
pub struct OrderInfo {
    id: String,
    price: String,
    size: String,
    product_id: String,
    side: String,
    stp: Option<String>,
    r#type: String,
    time_in_force: String,
    post_only: bool,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
    fill_fees: String,
    filled_size: String,
    executed_value: String,
    status: String,
    settled: bool,
}

/// A structure that repersents Report Info
#[derive(Debug, Deserialize)]
pub struct ReportInfo {
    id: String,
    r#type: String,
    status: String,
    #[serde(default, deserialize_with = "deserialize_option_to_date")]
    created_at: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_option_to_date")]
    completed_at: Option<DateTime<Utc>>,
    #[serde(default, deserialize_with = "deserialize_option_to_date")]
    expires_at: Option<DateTime<Utc>>,
    file_url: Option<String>,
    params: Option<ReportParams>,
}

/// A structure that repersents Report Info Params
#[derive(Debug, Deserialize)]
pub struct ReportParams {
    #[serde(deserialize_with = "deserialize_to_date")]
    start_date: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_to_date")]
    end_date: DateTime<Utc>,
}

/// A structure that repersents a Fill
#[derive(Debug, Deserialize)]
pub struct Fill {
    trade_id: u64,
    product_id: String,
    price: String,
    size: String,
    order_id: String,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
    liquidity: String,
    fee: String,
    settled: bool,
    side: String,
}

/// A structure that represents your current maker & taker fee rates, as well as your 30-day trailing volume
#[derive(Debug, Deserialize)]
pub struct Fees {
    maker_fee_rate: String,
    taker_fee_rate: String,
    usd_volume: Option<String>,
}

/// A structure represents a single profile
#[derive(Debug, Deserialize)]
pub struct Profile {
    id: String,
    user_id: String,
    name: String,
    active: bool,
    is_default: bool,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
}
