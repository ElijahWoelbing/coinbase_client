//! A library for the Coinbase Pro [API](https://docs.pro.coinbase.com/).
//!   
//! **Coinbase Client** is separated into two categories: `PrivateClient` and `PublicClient`. `PrivateClient` requires authentication and provide access to placing orders and other account information. `PublicClient` provides market data and is public.
pub mod error;
pub mod private_client;
pub mod public_client;

use self::error::{Error, ErrorKind, ErrorMessage, StatusError};
use chrono::{DateTime, TimeZone, Utc};
use serde::{self, de};
use serde::{Deserialize, Deserializer};

pub(crate) const COINBASE_API_URL: &'static str = "https://api.pro.coinbase.com";
pub(crate) const COINBASE_SANDBOX_API_URL: &'static str =
    "https://api-public.sandbox.pro.coinbase.com";

/// alias for serde_json::Value return type for data that cannot predictably deserialized into a strongly typed struct
pub type Json = serde_json::Value;

// derserilize to a type that impls the Deserialize trait
pub(crate) async fn deserialize_response<T>(response: reqwest::Response) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let error_message = response.json::<ErrorMessage>().await?;
        return Err(Error::new(ErrorKind::Status(StatusError::new(
            status.as_u16(),
            error_message.message,
        ))));
    }

    Ok(response.json::<T>().await?)
}

// deserializes a ISO 8601 / RFC 3339 date & time format str to a DateTime<Utc>
pub(crate) fn deserialize_to_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Utc.datetime_from_str(&s, "%+")
        .map_err(serde::de::Error::custom)
}

pub(crate) fn deserialize_option_to_date<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "deserialize_to_date")] DateTime<Utc>);
    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

pub(crate) fn configure_pagination(
    before: Option<&str>,
    after: Option<&str>,
    limit: Option<u16>,
) -> String {
    match (before, after, limit) {
        (None, None, None) => String::from(""),
        (None, None, Some(l)) => format!("limit={}", l),
        (None, Some(a), None) => format!("after={}", a),
        (Some(b), None, None) => format!("before={}", b),
        (None, Some(a), Some(l)) => format!("after={}&limit={}", a, l),
        (Some(b), None, Some(l)) => format!("before={}&limit={}", b, l),
        (Some(b), Some(a), None) => format!("before={}&after={}", b, a),
        (Some(b), Some(a), Some(l)) => format!("before={}&after={}&limit={}", b, a, l),
    }
}
