use serde::{Serialize, Deserialize};

/// Average Price Stream Data Structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AveragePrice {
    #[serde(rename = "e")]
    pub event_type: String, // Event type

    #[serde(rename = "E")]
    pub event_time: u64, // Event time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "i")]
    pub average_price_interval: String, // Average price interval

    #[serde(rename = "w")]
    pub average_price: String, // Average price

    #[serde(rename = "T")]
    pub last_trade_time: u64, // Last trade time
}
