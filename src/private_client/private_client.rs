use crate::error::{Error, ErrorKind};
use crate::json::*;
use base64;
use crypto::{self, mac::Mac};
use reqwest::{self, header::HeaderMap};
use serde;
use std::time::{SystemTime, SystemTimeError};

const COINBASE_API_URL: &str = "https://api-public.sandbox.pro.coinbase.com"; // sandbox url

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
        T: serde::de::DeserializeOwned,
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
            return Err(Error::new(ErrorKind::Status(status)));
        }
        let json = res.json().await?;
        Ok(json)
    }

    async fn post_and_deserialize<T, K>(&self, path: &str, body: K) -> Result<T, Error>
    where
        K: serde::Serialize,            // body must serialize
        T: serde::de::DeserializeOwned, // response must deserialize
    {
        let body_string = serde_json::to_string(&body)?;
        println!("{}", body_string);
        let headers = self.access_headers(path, Some(&body_string), "POST");
        let url = format!("{}{}", COINBASE_API_URL, path);
        let res = self
            .reqwest_client
            .post(url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;
        let status = res.status();
        if !status.is_success() {
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

    pub async fn place_order(&self, order: Order) -> Result<OrderResponse, Error> {
        Ok(self.post_and_deserialize("/orders", order).await?)
    }
}
