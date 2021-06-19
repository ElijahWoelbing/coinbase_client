use crate::configure_pagination;
use crate::{
    deserialize_option_to_date, deserialize_response, deserialize_to_date, deserialize_to_f64,
    COINBASE_API_URL, COINBASE_SANDBOX_API_URL,
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

/// alias for serde_json::Value used for data that cannot predictably be turned into its own struct
pub type Json = serde_json::Value;

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
        let params = configure_pagination(before, after, limit);
        println!("{}{}", path, params);
        self.get(&format!("{}{}", path, params)).await
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

    // deserialize to type T
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

    /// get trading account by account ID
    pub async fn get_account(&self, account_id: &str) -> Result<Account, Error> {
        let account = self.get(&format!("/accounts/{}", account_id)).await?;
        Ok(account)
    }

    /// List account activity of the API key's profile.
    /// <br>
    /// Account activity either increases or decreases your account balance.
    /// <br>
    /// Items are paginated and sorted latest first
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
    /// Holds are placed on an account for any active orders or pending withdraw requests.
    /// <br>
    /// As an order is filled, the hold amount is updated. If an order is canceled, any remaining hold is removed.
    /// <br>
    /// For a withdraw, once it is completed, the hold is removed.
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

    /// get recent fills by specified product_id of the API key's profile
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

    /// get information on your payment method transfer limits, as well as buy/sell limits per currency
    pub async fn get_limits(&self) -> Result<Json, Error> {
        Ok(self.get(&format!("/users/self/exchange-limits")).await?)
    }

    /// get deposits from the profile of the API key, in descending order by created time
    /// <br>
    /// **optional parameters**
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
    /// get internal deposits from the profile of the API key, in descending order by created time
    /// <br>
    /// **optional parameters**
    /// *profile_id*: limit list of internal deposits to this profile_id. By default, it retrieves internal deposits using default profile
    /// <br>
    /// <br>
    /// *before*: if before is set, then it returns internal deposits created after the before timestamp, sorted by oldest creation date
    /// <br>
    /// <br>
    /// *after*: if after is set, then it returns internal deposits created before the after timestamp, sorted by newest
    /// <br>
    /// <br>
    /// *limit*: truncate list to this many internal deposits, capped at 100. Default is 100.
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

    /// get information on a single deposit
    pub async fn get_deposit(&self, transfer_id: &str) -> Result<Json, Error> {
        Ok(self.get(&format!("/transfers/{}", transfer_id)).await?)
    }

    /// get your payment methods
    pub async fn get_payment_methods(&self) -> Result<Json, Error> {
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
    pub async fn get_coinbase_accounts(&self) -> Result<Json, Error> {
        Ok(self.get("/coinbase-accounts").await?)
    }

    /// generate an address for crypto deposits
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
    /// get information on a single withdrawal
    pub async fn get_withdrawl(&self, transfer_id: &str) -> Result<Json, Error> {
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
    ) -> Result<Json, Error> {
        Ok(self
            .post_and_deserialize(
                "/withdrawals/coinbase-account",
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
    pub async fn oracle(&self) -> Result<Json, Error> {
        Ok(self.get("/oracle").await?)
    }
}

pub enum OrderStatus {
    Open,
    Active,
    Pending,
    OpenActive,
    OpenPending,
    ActivePending,
    OpenActivePending,
}

/// Stablecoin Conversion
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

/// Account
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

/// Account History
#[derive(Deserialize, Debug)]
pub struct AccountHistory {
    id: String,
    #[serde(deserialize_with = "deserialize_to_date")]
    created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_to_f64")]
    amount: f64,
    #[serde(deserialize_with = "deserialize_to_f64")]
    balance: f64,
    r#type: String,
    details: AccountHistoryDetails,
}

/// Account Hold
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

#[derive(Deserialize, Debug)]
pub struct AccountHistoryDetails {
    order_id: Option<String>,
    trade_id: Option<String>,
    product_id: Option<String>,
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
