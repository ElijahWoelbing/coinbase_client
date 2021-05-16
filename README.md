# Coinbase Client [![Latest Version]][crates.io]
Rust library for Coinbase's Pro API.

[Documentation](https://docs.rs/coinbase-client/0.1.1/coinbase_client/)

[Latest Version]: https://img.shields.io/badge/Version-0.1.1-green
[crates.io]: https://crates.io/crates/coinbase_client


**Usage**

Requires [Tokio](https://github.com/tokio-rs/tokio) runtime
```
use coinbase_client::public_client::PublicClient;
use futures;

#[tokio::main] 
async fn main() {
    let client: PublicClient = PublicClient::new();
    let future = client.get_products();
    let json: Vev<Product> = futures::executor::block_on(future).unwrap();
}
```
