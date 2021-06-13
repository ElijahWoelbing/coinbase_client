//! A library for the Coinbase Pro [API](https://docs.pro.coinbase.com/).
//!   
//! **Coinbase Client** is separated into two categories: `PrivateClient` and `PublicClient`. `PrivateClient` requires authentication and provide access to placing orders and other account information. `PublicClient` provides market data and is public.
pub mod error;
pub mod private_client;
pub mod public_client;

use self::error::{Error, ErrorKind, ErrorMessage, StatusError};
use serde::{self, de};
use std::fmt;

pub(crate) const COINBASE_API_URL: &'static str = "https://api.pro.coinbase.com";

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

struct F64visitor;

impl<'de> de::Visitor<'de> for F64visitor {
    type Value = f64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "This Visitor expects to receive a str or string that will parse to a float"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<f64>() {
            Ok(n) => Ok(n),
            Err(e) => Err(E::custom(format!("Parse error {} for {}", e, v))),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<f64>() {
            Ok(n) => Ok(n),
            Err(e) => Err(E::custom(format!("Parse error {} for {}", e, v))),
        }
    }
}

/// deserializes a f64 compatable str or string to a f64
pub(crate) fn deserialize_f64<'de, D>(d: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    d.deserialize_str(F64visitor)
}
