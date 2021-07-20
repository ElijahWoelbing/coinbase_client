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
        .get_account("0589d87c-154d-4f5b-9ed9-ff814f70e04a")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_place_order_market_funds() {
    let order = OrderBuilder::market(
        OrderSide::Buy,
        "BTC-USD",
        SizeOrFunds::Funds("10.00".to_owned()),
    )
    .build();
    let client = create_client();
    let _res = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_place_order_market_size() {
    let order =
        OrderBuilder::market(OrderSide::Buy, "ADA", SizeOrFunds::Size("5.00".to_owned())).build();
    let client = create_client();
    let _res = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_place_order_limit() {
    let order = Order::limit_builder(OrderSide::Buy, "BTC-USD", "36000.0", "1.0").build();
    let client = create_client();
    let _res = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_place_order_stop() {
    let order = OrderBuilder::stop(
        OrderSide::Buy,
        "BTC-USD",
        "36000.0",
        "1.0",
        "37000.0",
        OrderStop::Loss,
    )
    .build();
    let client = create_client();
    let _res = client.place_order(order).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_cancel_order() {
    let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", "33000.0", "1.0").build();
    let client = create_client();
    let order_to_cancel_id = client.place_order(order).await.unwrap();
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
    let _orders = client
        .get_orders(
            Some(OrderStatus::OpenActivePending),
            Some("2021-06-19T20:24:20.467086Z"),
            None,
            None,
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_order() {
    let order = OrderBuilder::limit(OrderSide::Buy, "BTC-USD", "36000.0", "1.0").build();
    let client = create_client();
    let order_id = client.place_order(order).await.unwrap();
    let _order = client.get_order(&order_id).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_fill_by_order_id() {
    let client = create_client();
    let _fills = client
        .get_fill_by_order_id("4f2756cf-dcb5-492b-83e5-5f2141892758", None, None, None)
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_fills_by_product_id() {
    let product_id = "BTC-USD";
    let client = create_client();
    let _fills = client
        .get_fills_by_product_id(&product_id, None, None, None)
        .await
        .unwrap();
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
            Some("b7482eaa-3eea-4065-9d81-1484257c5f92"),
            None,
            None,
            None,
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_internal_deposits() {
    let client = create_client();
    let _deposits = client
        .get_internal_deposits(
            Some("e1d7731f-b7e2-4285-b711-eeec76fc2aff"),
            None,
            None,
            None,
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_deposit() {
    let client = create_client();
    let _deposit = client
        .get_deposit("80259339-7bf9-498f-8210-ddbd32a1c545")
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
        .deposit_funds("10.00", "USD", "9da3e279-20a1-57e4-95f8-52ec41041999")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_deposit_from_coinbase() {
    let client = create_client();
    let _deposit = client
        .deposit_funds_from_coinbase(13.468564, "ALGO", "2141660b-da3d-5060-8af1-b8478cf6dd44")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_generate_crypto_address() {
    let client = create_client();
    let _address = client
        .generate_crypto_deposit_address("2141660b-da3d-5060-8af1-b8478cf6dd44")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_withdrawls() {
    let client = create_client();
    let _withdrawls = client
        .get_withdrawls(
            Some("b7482eaa-3eea-4065-9d81-1484257c5f92"),
            None,
            None,
            None,
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_internal_withdrawls() {
    let client = create_client();
    let _withdrawls = client
        .get_internal_withdrawls(
            Some("b7482eaa-3eea-4065-9d81-1484257c5f92"),
            None,
            None,
            None,
        )
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_withdrawl() {
    let client = create_client();
    let _withdrawl = client
        .get_withdrawl("0e94a87f-9d50-4ead-86ac-7898830c5edf")
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_account_history() {
    let client = create_client();
    let _history = client
        .get_account_history("680f85f4-1a99-4108-93ce-a9066f9de246", None, None, Some(2))
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_account_holds() {
    let client = create_client();
    let _holds = client
        .get_account_holds(
            "680f85f4-1a99-4108-93ce-a9066f9de246",
            None,
            None,
            Some(100),
        )
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
        .get_profile("6e14a84c-610b-4c63-8b69-443920dffcaf")
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
            10.00,
        )
        .await
        .unwrap();
    assert_eq!(ok, "OK")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_to_crypto_address() {
    let client = create_client();
    let _res = client.withdraw_to_crypto_address(6.0, "ADA", "addr1qyk0yr3ht9d6hcqwp8q8j38nxs04npyjauzz9wp5jcfr95h64lvegfk57zmzltj3nmpjff6490ayyvjh0g6sne6hm3hspnnscy", None, None, None).await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_withdraw_to_coinbase() {
    let client = create_client();
    let _res = client
        .withdraw_to_coinbase(1.0, "ADA", "91bdfea7-f243-5baa-bb0d-5b93c9f09ffc")
        .await
        .unwrap();
}
