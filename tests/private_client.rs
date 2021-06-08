use coinbase_client::private_client::*;
use dotenv;
use std::env;

fn create_client() -> PrivateClient {
    dotenv::from_filename(".env").expect("error reading .env file");
    let secret = env::var("SECRET").expect("Cant find api secret");
    let passphrase = env::var("PASSPHRASE").expect("Cant find api passphrase");
    let key = env::var("KEY").expect("Cant find api key");
    PrivateClient::new_sandbox(secret, passphrase, key)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_accounts() {
    let client = create_client();
    let _json = client.get_accounts().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_market_funds() {
    let order = OrderBuilder::market(OrderSide::Buy, "BTC-USD", SizeOrFunds::Funds(10.00)).build();
    let client = create_client();
    let _json = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_market_size() {
    let order = OrderBuilder::market(OrderSide::Buy, "BTC-USD", SizeOrFunds::Size(0.02)).build();
    let client = create_client();
    let _json = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_limit() {
    let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", 36000.0, 1.0).build();
    let client = create_client();
    let _json = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_place_order_stop() {
    let order = OrderBuilder::stop(
        OrderSide::Buy,
        "BTC-USD",
        36000.0,
        1.0,
        37000.0,
        OrderStop::Loss,
    )
    .build();
    let client = create_client();
    let _json = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_cancel_order() {
    let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", 36000.0, 1.0).build();
    let client = create_client();
    // place order
    let order_to_cancel_id = client.place_order(order).await.unwrap();
    // cancel order
    let _canceled_order_id = client.cancel_order(&order_to_cancel_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_cancel_orders() {
    let client = create_client();
    let _canceled_orders_ids = client.cancel_orders().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_get_orders() {
    let client = create_client();
    let _orders = client.get_orders().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_order() {
    let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", 36000.0, 1.0).build();
    let client = create_client();
    // place order
    let order_id = client.place_order(order).await.unwrap();
    let _order = client.get_order(&order_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_fills_by_order_id() {
    let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", 36000.0, 1.0).build();
    let client = create_client();
    // place order
    let order_id = client.place_order(order).await.unwrap();

    let _fills = client.get_fills_by_order_id(&order_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_fills_by_product_id() {
    let product_id = "BTC-USD";
    let client = create_client();
    let _fills = client.get_fills_by_product_id(&product_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_limits() {
    let client = create_client();
    let limits = client.get_limits().await.unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_get_deposits() {
    let client = create_client();
    let limits = client
        .get_deposits(
            Some(DepositType::InternalDeposite),
            Some("f9783e6f-1874-402c-80dd-7eb1b323e23e"),
            Some(BeforeOrAfter::Before),
            Some(1),
        )
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]

async fn test_get_deposit() {
    let client = create_client();
    let limits = client
        .get_deposit("80259339-7bf9-498f-8200-ddbd32a1c545")
        .await;
}
