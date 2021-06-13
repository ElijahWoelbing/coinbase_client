# Coinbase Client [![Latest Version]][crates.io]
Rust library for Coinbase's Pro API.

[Documentation](https://docs.rs/coinbase-client/0.1.1/coinbase_client/)

[Latest Version]: https://img.shields.io/badge/Version-0.1.2-green
[crates.io]: https://crates.io/crates/coinbase_client


**Usage**

Requires [Tokio](https://github.com/tokio-rs/tokio) runtime
```
use coinbase_client::private_client::*;

// placing a market order
#[tokio::main] 
async fn main() {
    let client = PrivateClient::new("YOUR_API_SECRET", "YOUR_API_PASSPHRASE", "YOUR_API_KEY")
    let order = OrderBuilder::market(OrderSide::Buy, "BTC-USD", SizeOrFunds::Size(0.02)).build();
    let order_id = client.place_order(order).await.expect("unable to place order");
}
```
