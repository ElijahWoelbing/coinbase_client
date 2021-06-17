use coinbase_client::public_client::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_products() {
    let client = PublicClient::new_sandbox();
    let _products = client.get_products().await.unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product() {
    let client = PublicClient::new_sandbox();
    let _product = client.get_product("BTC-USD").await.unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book_all() {
    let client = PublicClient::new_sandbox();
    let _order_book = client.get_product_order_book_all("BTC-USD").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book_top50() {
    let client = PublicClient::new_sandbox();
    let _order_book = client
        .get_product_order_book_top50("BTC-USD")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book() {
    let client = PublicClient::new_sandbox();
    let _order_book = client.get_product_order_book("BTC-USD").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_ticker() {
    let client = PublicClient::new_sandbox();
    let _ticker = client.get_product_ticker("BTC-USD").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_trades() {
    let client = PublicClient::new_sandbox();
    let _trades = client
        .get_product_trades("BTC-USD", Some(83162), Some(83173), Some(100))
        .await
        .unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_historic_rates() {
    let client = PublicClient::new_sandbox();
    let _historical_rates = client
        .get_product_historic_rates("BTC-USD", None, None, Some(Granularity::OneMinute))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_24hr_stats() {
    let client = PublicClient::new_sandbox();
    let _twenty_four_hour_stats = client.get_product_24hr_stats("BTC-USD").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_currencies() {
    let client = PublicClient::new_sandbox();
    let _currencies = client.get_currencies().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_currency() {
    let client = PublicClient::new_sandbox();
    let _currency = client.get_currency("BTC").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_time() {
    let client = PublicClient::new_sandbox();
    let _time = client.get_time().await.unwrap();
}
