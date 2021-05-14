use super::public_client::PublicClient;
#[cfg(test)]

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_products() {
    let client = PublicClient::new();
    let future = client.get_products();
    let json = futures::executor::block_on(future).unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product() {
    let client = PublicClient::new();
    let future = client.get_product("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book_all() {
    let client = PublicClient::new();
    let future = client.get_product_order_book_all("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book_top50() {
    let client = PublicClient::new();
    let future = client.get_product_order_book_top50("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_order_book() {
    let client = PublicClient::new();
    let future = client.get_product_order_book("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_ticker() {
    let client = PublicClient::new();
    let future = client.get_product_ticker("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_trades() {
    let client = PublicClient::new();
    let future = client.get_product_trades("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_historic_rates() {
    let client = PublicClient::new();
    let future = client.get_product_historic_rates("MIR-EUR");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_product_24hr_stats() {
    let client = PublicClient::new();
    let future = client.get_product_24hr_stats("BTC-USD");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_currencies() {
    let client = PublicClient::new();
    let future = client.get_currencies();
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_currency() {
    let client = PublicClient::new();
    let future = client.get_currency("BTC");
    let json = futures::executor::block_on(future).unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_time() {
    let client = PublicClient::new();
    let future = client.get_time();
    let json = futures::executor::block_on(future).unwrap();
}