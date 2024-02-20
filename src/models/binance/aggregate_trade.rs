use serde::{Serialize, Deserialize};

/// Aggregate Trade Streams Data Structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregateTrade {
    #[serde(rename = "e")]
    pub event_type: String, // Event type

    #[serde(rename = "E")]
    pub event_time: u64, // Event time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "a")]
    pub aggregate_trade_id: u64, // Aggregate trade ID

    #[serde(rename = "p")]
    pub price: String, // Price

    #[serde(rename = "q")]
    pub quantity: String, // Quantity

    #[serde(rename = "f")]
    pub first_trade_id: u64, // First trade ID

    #[serde(rename = "l")]
    pub last_trade_id: u64, // Last trade ID

    #[serde(rename = "T")]
    pub trade_time: u64, // Trade time

    #[serde(rename = "m")]
    pub is_buyer_market_maker: bool, // Is the buyer the market maker?

    #[serde(rename = "M")]
    pub ignore: bool, // Ignore
}
