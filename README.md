# Coinbase Client [![Latest Version]][crates.io]
Rust library for Coinbase's Pro API.

[Documentation](https://docs.rs/coinbase-client/0.1.1/coinbase_client/)

[Latest Version]: https://img.shields.io/badge/Version-0.1.1-green
[crates.io]: https://crates.io/crates/coinbase_client


**Usage**

Requires [Tokio](https://github.com/tokio-rs/tokio) runtime
```
use coinbase_client::public_client::PublicClient;

#[tokio::main] 
async fn main() {
    let secret = "SECRET";
    let passphrase = "PASSPHRASE";
    let key = "KEY";
    let client = PrivateClient::new(secret, passphrase, key)
    let order = OrderBuilder::market(OrderSide::Buy, "BTC-USD", SizeOrFunds::Size(0.02)).build();
    let order_id = client.place_order(order).await.unwrap();
}
```
