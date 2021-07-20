use serde::Serialize;

/// A `OrderBuilder` should be used to create a `Order` with  custom configuration.
#[derive(Serialize, Debug)]
pub struct Order {
    r#type: String,
    size: Option<f64>,
    price: Option<f64>,
    side: OrderSide,
    client_oid: Option<String>,
    self_trade_prevention: Option<SelfTradePrevention>,
    time_in_force: Option<TimeInForce>,
    cancel_after: Option<CancelAfter>,
    post_only: Option<bool>,
    funds: Option<f64>,
    product_id: String,
    stp: Option<String>,
    stop: Option<OrderStop>,
    stop_price: Option<f64>,
}

/// A `OrderBuilder` should be used to create a `Order` with  custom configuration.
impl Order {
    /// returns a `OrderBuilder` with required market-order parameters, equivalent OrderBuilder::market
    pub fn market_builder(
        side: OrderSide,
        product_id: &str,
        size_or_funds: SizeOrFunds,
    ) -> impl SharedOptions {
        OrderBuilder {
            r#type: "market".to_string(),
            size: match size_or_funds {
                SizeOrFunds::Size(n) => Some(n),
                _ => None,
            },
            price: None,
            side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: match size_or_funds {
                SizeOrFunds::Funds(n) => Some(n),
                _ => None,
            },
            product_id: product_id.to_string(),
            stp: None,
            stop: None,
            stop_price: None,
        }
    }

    /// returns a `OrderBuilder` with required limit-order parameters, equivalent OrderBuilder::limit
    pub fn limit_builder(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
    ) -> impl LimitOptions + SharedOptions {
        OrderBuilder {
            r#type: "limit".to_string(),
            size: Some(size),
            price: Some(price),
            side: side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: None,
            product_id: product_id.to_string(),
            stp: None,
            stop: None,
            stop_price: None,
        }
    }

    /// returns a `OrderBuilder` with required stop-order parameters, equivalent OrderBuilder::stop
    pub fn stop_builder(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
        stop_price: f64,
        stop: OrderStop,
    ) -> impl SharedOptions {
        OrderBuilder {
            r#type: "limit".to_string(),
            size: Some(size),
            price: Some(price),
            side: side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: None,
            product_id: product_id.to_string(),
            stp: None,
            stop: Some(stop),
            stop_price: Some(stop_price),
        }
    }
}

/// A `OrderBuilder` can be used to create a `Order` with custom configuration.
/// <br>
/// Configuration parameters details can be found [here](https://docs.pro.coinbase.com/#orders)
pub struct OrderBuilder {
    r#type: String,
    size: Option<f64>,
    price: Option<f64>,
    side: OrderSide,
    client_oid: Option<String>,
    self_trade_prevention: Option<SelfTradePrevention>,
    time_in_force: Option<TimeInForce>,
    cancel_after: Option<CancelAfter>,
    post_only: Option<bool>,
    funds: Option<f64>,
    product_id: String,
    stp: Option<String>,
    stop: Option<OrderStop>,
    stop_price: Option<f64>,
}

impl OrderBuilder {
    /// returns a `OrderBuilder` with required market-order parameters.
    pub fn market(
        side: OrderSide,
        product_id: &str,
        size_or_funds: SizeOrFunds,
    ) -> impl SharedOptions {
        Self {
            r#type: "market".to_string(),
            size: match size_or_funds {
                SizeOrFunds::Size(n) => Some(n),
                _ => None,
            },
            price: None,
            side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: match size_or_funds {
                SizeOrFunds::Funds(n) => Some(n),
                _ => None,
            },
            product_id: product_id.to_string(),
            stp: None,
            stop: None,
            stop_price: None,
        }
    }

    /// returns a `OrderBuilder` with required limit-order parameters.
    pub fn limit(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
    ) -> impl LimitOptions + SharedOptions {
        Self {
            r#type: "limit".to_string(),
            size: Some(size),
            price: Some(price),
            side: side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: None,
            product_id: product_id.to_string(),
            stp: None,
            stop: None,
            stop_price: None,
        }
    }

    /// returns a `OrderBuilder` with required stop-order parameters.
    pub fn stop(
        side: OrderSide,
        product_id: &str,
        price: f64,
        size: f64,
        stop_price: f64,
        stop: OrderStop,
    ) -> impl SharedOptions {
        Self {
            r#type: "limit".to_string(),
            size: Some(size),
            price: Some(price),
            side: side,
            client_oid: None,
            self_trade_prevention: None,
            time_in_force: None,
            cancel_after: None,
            post_only: None,
            funds: None,
            product_id: product_id.to_string(),
            stp: None,
            stop: Some(stop),
            stop_price: Some(stop_price),
        }
    }
}

/// 'SharedOptions' options can be used with market, limit and stop order types
pub trait SharedOptions {
    fn self_trade_prevention(self, self_trade_prevention: SelfTradePrevention) -> Self;
    fn client_oid(self, client_oid: String) -> Self;
    fn build(self) -> Order;
}

impl SharedOptions for OrderBuilder {
    /// Sets the Orders self-trade behavior
    fn self_trade_prevention(mut self, self_trade_prevention: SelfTradePrevention) -> Self {
        self.self_trade_prevention = Some(self_trade_prevention);
        self
    }

    /// Sets the Order ID to identify your order
    /// The client_oid is different than the server-assigned order id.
    /// <br>
    /// If you are consuming the public feed and see a received message with your client_oid,
    /// <br>
    /// you should record the server-assigned order_id as it will be used for future order status updates.
    /// <br>
    /// The client_oid will NOT be used after the received message is sent.
    fn client_oid(mut self, client_oid: String) -> Self {
        self.client_oid = Some(client_oid);
        self
    }

    /// Builds `Order`
    fn build(self) -> Order {
        Order {
            r#type: self.r#type,
            size: self.size,
            price: self.price,
            side: self.side,
            client_oid: self.client_oid,
            self_trade_prevention: self.self_trade_prevention,
            time_in_force: self.time_in_force,
            cancel_after: self.cancel_after,
            post_only: self.post_only,
            funds: self.funds,
            product_id: self.product_id,
            stp: self.stp,
            stop: self.stop,
            stop_price: self.stop_price,
        }
    }
}

/// Builder options for Limit Orders
pub trait LimitOptions {
    fn time_in_force(self, time_in_force: TimeInForce) -> Self;
}

impl LimitOptions for OrderBuilder {
    /// This option provides guarantees about the lifetime of an Order
    fn time_in_force(mut self, time_in_force: TimeInForce) -> Self {
        match time_in_force {
            TimeInForce::GoodTillTime {
                cancel_after,
                post_only,
            } => {
                self.cancel_after = Some(cancel_after);
                self.post_only = Some(post_only);
            }
            TimeInForce::GoodTillCancel { post_only } => self.post_only = Some(post_only),
            _ => {}
        }
        self.time_in_force = Some(time_in_force);
        self
    }
}

/// Buy or Sell `Order`
#[derive(Clone, Copy, Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}
/// Loss triggers when the last trade price changes to a value at or below the stop_price.
/// <br>
/// Entry triggers when the last trade price changes to a value at or above the stop_price.
#[derive(Clone, Copy, Debug)]
pub enum OrderStop {
    Loss,
    Entry,
}

/// Size or Funds of Currency
#[derive(Clone, Copy, Debug)]
pub enum SizeOrFunds {
    Size(f64),
    Funds(f64),
}

// Time in force policies provide guarantees about the lifetime of an `Order`
#[derive(Clone, Copy, Debug)]
pub enum TimeInForce {
    GoodTillCancel {
        post_only: bool,
    },
    GoodTillTime {
        cancel_after: CancelAfter,
        post_only: bool,
    },
    ImmediateOrCancel,
    FillOrKill,
}

#[derive(Clone, Copy, Debug)]
pub enum CancelAfter {
    Minute,
    Hour,
    Day,
}

/// Used to change the self-trade behavior
#[derive(Clone, Copy, Debug)]
pub enum SelfTradePrevention {
    DecreaseCancel,
    CancelOldest,
    CancelNewest,
    CancelBoth,
}

impl serde::Serialize for SelfTradePrevention {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::CancelBoth => serializer.serialize_str("cb"),
            Self::DecreaseCancel => serializer.serialize_str("dc"),
            Self::CancelOldest => serializer.serialize_str("co"),
            Self::CancelNewest => serializer.serialize_str("cn"),
        }
    }
}

impl serde::Serialize for OrderStop {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Loss => serializer.serialize_str("loss"),
            Self::Entry => serializer.serialize_str("entry"),
        }
    }
}

impl serde::Serialize for TimeInForce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::GoodTillCancel { post_only: _ } => serializer.serialize_str("GTC"),
            Self::GoodTillTime {
                cancel_after: _,
                post_only: _,
            } => serializer.serialize_str("GTT"),
            Self::ImmediateOrCancel => serializer.serialize_str("IOC"),
            Self::FillOrKill => serializer.serialize_str("FOK"),
        }
    }
}

impl serde::Serialize for CancelAfter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Minute => serializer.serialize_str("min"),
            Self::Hour => serializer.serialize_str("hour"),
            Self::Day => serializer.serialize_str("day"),
        }
    }
}

impl serde::Serialize for OrderSide {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Self::Buy => serializer.serialize_str("buy"),
            Self::Sell => serializer.serialize_str("sell"),
        }
    }
}

impl serde::Serialize for SizeOrFunds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match *self {
            Self::Size(size) => serializer.serialize_f64(size),
            Self::Funds(funds) => serializer.serialize_f64(funds),
        }
    }
}
