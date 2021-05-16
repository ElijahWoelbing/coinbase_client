use super::error::{Error, ErrorKind};
use base64;
use chrono::format::format;
use crypto::{self, mac::Mac};
use futures;
use reqwest::{self, header::HeaderMap};
use serde::de::DeserializeOwned;
use std::time::{SystemTime, SystemTimeError};
use super::json::*;

const COINBASE_API_URL: &str = "https://api.pro.coinbase.com";

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

    async fn get_and_deserialize<T>(&self, url: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let res = self
            .reqwest_client
            .get(url)
            .headers(self.access_headers(url, None, "GET"))
            .send()
            .await?;
            let status = res.status();
            if !status.is_success() {
                return Err(Error::new(ErrorKind::Status(status)));
            }
            let json = res.json()
            .await?;
        Ok(json)
    }

    pub async fn get_accounts(&self) -> Result<Vec<Account>, Error> {
        let url = format!("{}/accounts", COINBASE_API_URL);
        let accounts = self.get_and_deserialize(&url).await?;
        Ok(accounts)
    }



    fn get_current_timestamp() -> Result<String, SystemTimeError> {
        Ok(SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            .to_string())
    }

    fn access_headers(
        &self,
        request_path: &str,
        body: Option<&str>,
        meathod: &str,
    ) -> reqwest::header::HeaderMap {
        let timestamp = PrivateClient::get_current_timestamp().unwrap();
        let signature = self.sign_message(request_path, body, &timestamp, meathod);
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
                .expect("invalid cb-access-sign header value"),
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
        request_path: &str,
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
                prehash.push_str(&request_path);
                prehash.push_str(&body);
            }
            None => {
                prehash.push_str(&timestamp);
                prehash.push_str(&meathod);
                prehash.push_str(&request_path);
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