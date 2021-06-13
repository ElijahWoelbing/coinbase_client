use super::{
    deserialize_option_to_date, deserialize_response, deserialize_to_date, deserialize_to_f64,
    COINBASE_API_URL, COINBASE_SANDBOX_API_URL,
};
use crate::error::{Error, ErrorKind, ErrorMessage, StatusError};
use base64;
use chrono::{DateTime, Utc};
use core::f64;
use crypto::{self, mac::Mac};
use reqwest;
use serde::{self, Deserialize, Serialize};
use std::{
    slice::RSplit,
    time::{SystemTime, SystemTimeError},
};

/// alias for serde_json::Value used for data that cannot predictably be turned into its own struct
pub type JsonValue = serde_json::Value;

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
            secret,
            key,
            passphrase,
            url: COINBASE_SANDBOX_API_URL,
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

    async fn post_and_deserialize<T, K>(&self, path: &str, body: Option<K>) -> Result<T, Error>
    where
        K: serde::Serialize,            // body must serialize
        T: serde::de::DeserializeOwned, // response must deserialize
    {
        deserialize_response::<T>(self.post(path, body).await?).await
    }

    async fn post<K>(&self, path: &str, body: Option<K>) -> Result<reqwest::Response, Error>
    where
        K: serde::Serialize, // body must serialize
    {
        let url = format!("{}{}", self.url, path);
        let request_builder = self.reqwest_client.post(url);
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
        #[derive(Deserialize, Debug)]
        pub struct OrderID {
            pub id: String,
        }
        Ok(self
            .post_and_deserialize::<OrderID, _>("/orders", Some(order))
            .await?
            .id)
    }

    /// cancel order specified by order ID
    pub async fn cancel_order(&self, order_id: &str) -> Result<String, Error> {
        Ok(self.delete(&format!("/orders/{}", order_id)).await?)
    }

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

    /// get information on your payment method transfer limits, as well as buy/sell limits per currency
    pub async fn get_limits(&self) -> Result<JsonValue, Error> {
        Ok(self.get(&format!("/users/self/exchange-limits")).await?)
    }

    /// get deposits from the profile of the API key, in descending order by created time
    /// <br>
    /// **optional parameters**
    /// *deposit_type*: set to deposit or internal_deposit (transfer between portfolios)
    /// <br>
    /// <br>
    /// *profile_id*: limit list of deposits to this profile_id. By default, it retrieves deposits using default profile
    /// <br>
    /// <br>
    /// *before*: if before is set, then it returns deposits created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// <br>
    /// *after*: if after is set, then it returns deposits created before the after timestamp, sorted by newest
    /// <br>
    /// <br>
    /// *limit*: truncate list to this many deposits, capped at 100. Default is 100.
    pub async fn get_deposits(
        &self,
        deposit_type: Option<DepositType>,
        profile_id: Option<&str>,
        before_or_after: Option<BeforeOrAfter>,
        limit: Option<u8>,
    ) -> Result<JsonValue, Error> {
        let mut url = String::from("/transfers/");
        let mut appended = false;
        if let Some(n) = deposit_type {
            appended = true;
            match n {
                DepositType::Deposit => url.push_str("?type=deposit"),
                DepositType::InternalDeposite => url.push_str("?type=internal_deposit"),
            }
        }
        if let Some(n) = profile_id {
            if appended == false {
                appended = true;
                url.push_str(&format!("?profile_id={}", n))
            } else {
                url.push_str(&format!("&profile_id={}", n));
            }
        }
        if let Some(n) = before_or_after {
            if appended == false {
                appended = true;
                url.push_str("?")
            } else {
                url.push_str("&");
            }
            match n {
                BeforeOrAfter::Before => url.push_str("before"),
                BeforeOrAfter::After => url.push_str("after"),
            }
        }
        if let Some(mut n) = limit {
            if n > 100 {
                n = 100;
            }
            if appended == false {
                url.push_str(&format!("?limit={}", n))
            } else {
                url.push_str(&format!("&limit={}", n));
            }
        }

        Ok(self.get(&url).await?)
    }

    /// get information on a single deposit
    pub async fn get_deposit(&self, transfer_id: &str) -> Result<JsonValue, Error> {
        Ok(self.get(&format!("/transfers/{}", transfer_id)).await?)
    }

    /// get your payment methods
    pub async fn get_payment_methods(&self) -> Result<JsonValue, Error> {
        Ok(self.get("/payment-methods").await?)
    }

    /// deposit funds from a payment method
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

    /// deposit funds from a coinbase account
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

    /// get a list of your coinbase accounts
    pub async fn get_coinbase_accounts(&self) -> Result<JsonValue, Error> {
        Ok(self.get("/coinbase-accounts").await?)
    }

    /// generate an address for crypto deposits
    pub async fn generate_crypto_deposit_address(
        &self,
        coinbase_account_id: &str,
    ) -> Result<JsonValue, Error> {
        Ok(self
            .post_and_deserialize::<_, JsonValue>(
                &format!("/coinbase-accounts/{}/addresses", coinbase_account_id),
                None,
            )
            .await?)
    }

    /// get withdrawals from the profile of the API key
    /// <br>
    /// **optional parameters**
    /// *withdrawl_type*: set to withdraw or internal_withdraw (transfer between portfolios)
    /// <br>
    /// <br>
    /// *profile_id*: limit list of withdrawals to this profile_id. By default, it retrieves withdrawals using default profile
    /// <br>
    /// <br>
    /// *before*: If before is set, then it returns withdrawals created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// <br>
    /// *after*: If after is set, then it returns withdrawals created before the after timestamp, sorted by newest
    /// <br>
    /// <br>
    /// *limit*: truncate list to this many withdrawals, capped at 100. Default is 100
    pub async fn get_withdrawls(
        &self,
        withdraw_type: Option<WithdrawType>,
        profile_id: Option<&str>,
        before_or_after: Option<BeforeOrAfter>,
        limit: Option<u8>,
    ) -> Result<JsonValue, Error> {
        let mut url = String::from("/transfers/");
        let mut appended = false;
        if let Some(n) = withdraw_type {
            appended = true;
            match n {
                WithdrawType::Withdraw => url.push_str("?type=withdraw"),
                WithdrawType::InternalWithdraw => url.push_str("?type=internal_withdraw"),
            }
        }
        if let Some(n) = profile_id {
            if appended == false {
                appended = true;
                url.push_str(&format!("?profile_id={}", n))
            } else {
                url.push_str(&format!("&profile_id={}", n));
            }
        }
        if let Some(n) = before_or_after {
            if appended == false {
                appended = true;
                url.push_str("?")
            } else {
                url.push_str("&");
            }
            match n {
                BeforeOrAfter::Before => url.push_str("before"),
                BeforeOrAfter::After => url.push_str("after"),
            }
        }
        if let Some(mut n) = limit {
            if n > 100 {
                n = 100;
            }
            if appended == false {
                url.push_str(&format!("?limit={}", n))
            } else {
                url.push_str(&format!("&limit={}", n));
            }
        }

        Ok(self.get(&url).await?)
    }

    /// get information on a single withdrawal
    pub async fn get_withdrawl(&self, transfer_id: &str) -> Result<JsonValue, Error> {
        Ok(self.get(&format!("/transfers/{}", transfer_id)).await?)
    }

    /// withdraw funds to a coinbase account
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

    /// withdraw funds to a crypto address.
    /// <br>
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
    pub async fn withdraw_to_crypto_address(
        &self,
        amount: f64,
        currency: &str,
        crypto_address: &str,
        destination_tag: Option<&str>,
        no_destination_tag: Option<bool>,
        add_network_fee_to_total: Option<bool>,
    ) -> Result<JsonValue, Error> {
        Ok(self
            .post_and_deserialize(
                "/withdrawals/coinbase-account",
                Some(serde_json::json!({
                        "amount": amount,
                        "currency": currency,
                        "crypto_address": crypto_address,

                })),
            )
            .await?)
    }

    /// get your current maker & taker fee rates, as well as your 30-day trailing volume
    pub async fn get_fees(&self) -> Result<Fees, Error> {
        Ok(self.get("/fees").await?)
    }

    /// get the network fee estimate when sending to the given address
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

    /// convert between stablecoins
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
    // creates a report
    //<br>
    //<br>
    /// reports provide batches of historic information about your profile in various human and machine readable forms
    pub async fn create_report<'a>(&self, report: Report) -> Result<ReportInfo, Error> {
        Ok(self.post_and_deserialize("/reports", Some(report)).await?)
    }

    /// get report status
    //<br>
    //<br>
    /// once a report request has been accepted for processing, the status becomes available
    pub async fn get_report(&self, report_id: &str) -> Result<ReportInfo, Error> {
        Ok(self.get(&format!("/reports/{}", report_id)).await?)
    }

    /// get your profiles
    pub async fn get_profiles(&self) -> Result<Vec<Profile>, Error> {
        Ok(self.get("/profiles").await?)
    }

    /// get a single profile by profile id
    pub async fn get_profile(&self, profile_id: &str) -> Result<Profile, Error> {
        Ok(self.get(&format!("/profiles/{}", profile_id)).await?)
    }

    /// transfer funds from API key's profile to another user owned profile
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

    /// get cryptographically signed prices ready to be posted on-chain using Open Oracle smart contracts.
    pub async fn oracle(&self) -> Result<JsonValue, Error> {
        Ok(self.get("/oracle").await?)
    }
}

#[derive(Debug, Deserialize)]
pub struct TransferStatus {
    status: String,
}

#[derive(serde::Serialize, Debug)]
pub struct Report {
    r#type: String,
    start_date: String,
    end_date: String,
    product_id: Option<String>,
    account_id: Option<String>,
    format: Format,
    email: Option<String>,
}

impl Report {
    pub fn fills_builder(
        start_date: &str,
        end_date: &str,
        product_id: &str,
    ) -> impl SharedReportOptions + FillsReportOptions {
        ReportBuilder {
            r#type: "fills".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: Some(product_id.to_string()),
            account_id: None,
            format: Format::PDF,
            email: None,
        }
    }

    pub fn account_builder(
        start_date: &str,
        end_date: &str,
        account_id: &str,
    ) -> impl SharedReportOptions + AccountReportOptions {
        ReportBuilder {
            r#type: "account".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: None,
            account_id: Some(account_id.to_string()),
            format: Format::PDF,
            email: None,
        }
    }
}

#[derive(Debug)]
pub struct ReportBuilder {
    r#type: String,
    start_date: String,
    end_date: String,
    product_id: Option<String>,
    account_id: Option<String>,
    format: Format,
    email: Option<String>,
}

impl ReportBuilder {
    pub fn fills(
        start_date: &str,
        end_date: &str,
        product_id: &str,
    ) -> impl SharedReportOptions + FillsReportOptions {
        Self {
            r#type: "fills".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: Some(product_id.to_string()),
            account_id: None,
            format: Format::PDF,
            email: None,
        }
    }

    pub fn account(
        start_date: &str,
        end_date: &str,
        account_id: &str,
    ) -> impl SharedReportOptions + AccountReportOptions {
        Self {
            r#type: "account".to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            product_id: None,
            account_id: Some(account_id.to_string()),
            format: Format::PDF,
            email: None,
        }
    }
}

pub trait FillsReportOptions {
    fn account_id(self, account_id: &str) -> Self;
}

impl FillsReportOptions for ReportBuilder {
    fn account_id(mut self, account_id: &str) -> Self {
        self.account_id = Some(account_id.to_string());
        self
    }
}
pub trait AccountReportOptions {
    fn product_id(self, product_id: &str) -> Self;
}

impl AccountReportOptions for ReportBuilder {
    fn product_id(mut self, product_id: &str) -> Self {
        self.product_id = Some(product_id.to_string());
        self
    }
}
pub trait SharedReportOptions {
    fn format(self, format: Format) -> Self;
    fn email(self, email: &str) -> Self;
    fn build(self) -> Report;
}

impl SharedReportOptions for ReportBuilder {
    fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    fn email(mut self, email: &str) -> Self {
        self.email = Some(email.to_string());
        self
    }

    fn build(self) -> Report {
        Report {
            r#type: self.r#type,
            start_date: self.start_date,
            end_date: self.end_date,
            product_id: self.product_id,
            account_id: self.account_id,
            format: self.format,
            email: self.email,
        }
    }
}

#[derive(Debug)]
pub enum Format {
    PDF,
    CSV,
}

pub enum WithdrawType {
    Withdraw,
    InternalWithdraw,
}

pub enum DepositType {
    Deposit,
    InternalDeposite,
}

pub enum BeforeOrAfter {
    Before,
    After,
}

#[derive(Deserialize, Debug)]
pub struct StablecoinConversion {
    id: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    amount: f64,
    from_account_id: String,
    to_account_id: String,
    from: String,
    to: String,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub currency: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub balance: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub available: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    pub hold: f64,
    pub profile_id: String,
    pub trading_enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct DepositInfo {
    id: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    amount: f64,
    currency: String,
    payout_at: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct WithdrawInfo {
    id: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    amount: f64,
    currency: String,
}

/// A `OrderBuilder` should be used to create a `Order` with  custom configuration.
#[derive(Serialize, Debug)]
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

impl Order {
    /// returns a `OrderBuilder` with requiered market-order parameters, equivalent OrderBuilder::market
    pub fn market_builder(
        side: OrderSide,
        product_id: &str,
        size_or_funds: SizeOrFunds,
    ) -> impl SharedOptions {
        OrderBuilder {
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

    /// returns a `OrderBuilder` with requiered limit-order parameters, equivalent OrderBuilder::limit
    pub fn limit_builder(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
    ) -> impl LimitOptions + SharedOptions {
        OrderBuilder {
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

    /// returns a `OrderBuilder` with requiered stop-order parameters, equivalent OrderBuilder::stop
    pub fn stop_builder(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
        stop_price: f64,
        stop: OrderStop,
    ) -> impl SharedOptions {
        OrderBuilder {
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
    /// returns a `OrderBuilder` with requiered market-order parameters.
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
    #[serde(deserialize_with = "deserialize_to_f64")]
    price: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    size: f64,
    product_id: String,
    side: String,
    stp: Option<String>,
    r#type: String,
    time_in_force: String,
    post_only: bool,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_to_f64")]
    fill_fees: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    filled_size: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    executed_value: f64,
    status: String,
    settled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Fill {
    trade_id: u64,
    product_id: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    price: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    size: f64,
    order_id: String,
    created_at: String,
    liquidity: String,
    #[serde(deserialize_with = "deserialize_to_f64")]
    fee: f64,
    settled: bool,
    side: String,
}

/// a structure that represents your current maker & taker fee rates, as well as your 30-day trailing volume
#[derive(Debug, Deserialize)]
pub struct Fees {
    #[serde(deserialize_with = "deserialize_to_f64")]
    maker_fee_rate: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    taker_fee_rate: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    usd_volume: f64,
}

/// a structure represents a single profile
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

#[derive(Debug, Deserialize)]
pub struct ReportParams {
    #[serde(deserialize_with = "deserialize_to_date")]
    start_date: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_to_date")]
    end_date: DateTime<Utc>,
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
            Self::CancelBoth => serializer.serialize_str("cb"),
            Self::DecreaseCancel => serializer.serialize_str("dc"),
            Self::CancelOldest => serializer.serialize_str("co"),
            Self::CancelNewest => serializer.serialize_str("cn"),
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
            Self::Buy => serializer.serialize_str("buy"),
            Self::Sell => serializer.serialize_str("sell"),
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

impl serde::Serialize for Format {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::PDF => serializer.serialize_str("pdf"),
            Self::CSV => serializer.serialize_str("csv"),
        }
    }
}
