use coinbase_client::{json::*, private_client::PrivateClient};
use dotenv;
use futures;
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
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_limit() {
    let order = Order::limit(
        0.01,
        100.0,
        OrderSide::Buy,
        "BTC-USD",
        TimeInForce::GoodTillTime {
            cancel_after: CancelAfter::Minute,
            post_only: false,
        },
    );
    let client = create_client();
    let future = client.place_order(order);
    let json = futures::executor::block_on(future).unwrap();
    println!("{:?}", json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_market_funds() {
    let order = Order::market(SizeOrFunds::Funds(10.0), OrderSide::Buy, "BTC-USD");
    let client = create_client();
    let future = client.place_order(order);
    let json = futures::executor::block_on(future).unwrap();
    println!("{:?}", json);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_market_size() {
    let order = Order::market(SizeOrFunds::Size(10.0), OrderSide::Buy, "BTC-USD");
    let client = create_client();
    let future = client.place_order(order);
    let json = futures::executor::block_on(future).unwrap();
    println!("{:?}", json);
}
