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
    let _accounts = client.get_accounts().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_account() {
    let client = create_client();
    let _account = client
        .get_account("1f6a7175-a89c-494f-986d-af9987e6dd69")
        .await
        .unwrap();
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
    let order = Order::limit_builder(OrderSide::Buy, "BTC-USD", 36000.0, 1.0).build();
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
    let _limits = client.get_limits().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_deposits() {
    let client = create_client();
    let _deposits = client
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
    let _limits = client
        .get_deposit("80259339-7bf9-498f-8200-ddbd32a1c545")
        .await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_payment_methods() {
    let client = create_client();
    let _payment_methods = client.get_payment_methods().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_coinbase_accounts() {
    let client = create_client();
    let _accounts = client.get_coinbase_accounts().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_deposit_funds() {
    let client = create_client();
    let _deposit = client
        .deposit_funds(10.00, "USD", "1b4b4fbc-8071-5e7c-b36e-a1c589a2cf20")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_deposit_from_coinbase() {
    let client = create_client();
    let _deposit = client
        .deposit_funds_from_coinbase(10.00, "BTC", "95671473-4dda-5264-a654-fc6923e8a334")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_generate_crypto_address() {
    let client = create_client();
    let _address = client
        .generate_crypto_deposit_address("95671473-4dda-5264-a654-fc6923e8a334")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_withdrawls() {
    let client = create_client();
    let _withdrawls = client
        .get_withdrawls(
            Some(WithdrawType::InternalWithdraw),
            Some("f9783e6f-1874-402c-80dd-7eb1b323e23e"),
            Some(BeforeOrAfter::After),
            Some(1),
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_withdrawl() {
    let client = create_client();
    let _withdrawl = client
        .get_withdrawl("80259339-7bf9-498f-8200-ddbd32a1c545")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_fees() {
    let client = create_client();
    let _fees = client.get_fees().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_fee_estimate_() {
    let client = create_client();
    let _fee = client
        .get_fee_estimate("ETH", "0x82289D45Ee8E806C63Ba0DC94a22d4238525d815")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_stablecoin_conversion() {
    let client = create_client();
    let _convertion = client
        .convert_stablecoin("USD", "USDC", 10.00)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_report() {
    let client = create_client();
    let report = Report::account_builder(
        "2014-11-01T00:00:00.000Z",
        "2021-06-11T02:48:15.853Z",
        "1f6a7175-a89c-494f-986d-af9987e6dd69",
    )
    .email("")
    .format(Format::CSV)
    .build();
    let _response = client.create_report(report).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_report() {
    let client = create_client();
    let _report = client
        .get_report("d4a3e847-b618-454d-bcb3-e77b0ad61600")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_profiles() {
    let client = create_client();
    let _profiles = client.get_profiles().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_profile() {
    let client = create_client();
    let _profile = client
        .get_profile("e1d7731f-b7e2-4285-b711-eeec76fc2aff")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_profile_transfer() {
    let client = create_client();
    let ok = client
        .create_profile_transfer(
            "e1d7731f-b7e2-4285-b711-eeec76fc2aff",
            "3510ac37-1a99-4c9c-9865-15f1bc5a832e",
            "USD",
            10000.00,
        )
        .await
        .unwrap();
    assert_eq!(ok, "OK")
}
