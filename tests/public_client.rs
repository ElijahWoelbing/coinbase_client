use coinbase_client::public_client::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_products() {
    let client = PublicClient::new();
    let _json = client.get_products().await.unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product() {
    let client = PublicClient::new();
    let _json = client.get_product("MIR-EUR").await.unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book_all() {
    let client = PublicClient::new();
    let _json = client.get_product_order_book_all("MIR-EUR").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book_top50() {
    let client = PublicClient::new();
    let _json = client
        .get_product_order_book_top50("MIR-EUR")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book() {
    let client = PublicClient::new();
    let _json = client.get_product_order_book("MIR-EUR").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_ticker() {
    let client = PublicClient::new();
    let _json = client.get_product_ticker("MIR-EUR").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_trades() {
    let client = PublicClient::new();
    let _json = client.get_product_trades("MIR-EUR").await.unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_historic_rates() {
    let client = PublicClient::new();
    let _json = client.get_product_historic_rates("MIR-EUR").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_24hr_stats() {
    let client = PublicClient::new();
    let _json = client.get_product_24hr_stats("BTC-USD").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_currencies() {
    let client = PublicClient::new();
    let _json = client.get_currencies().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_currency() {
    let client = PublicClient::new();
    let _json = client.get_currency("BTC").await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_time() {
    let client = PublicClient::new();
    let _json = client.get_time().await.unwrap();
}
