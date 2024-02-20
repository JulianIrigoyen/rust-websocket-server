use serde::{Serialize, Deserialize};

/// Trade Streams Data Structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trade {
    #[serde(rename = "e")]
    pub event_type: String, // Event type

    #[serde(rename = "E")]
    pub event_time: u64, // Event time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "t")]
    pub trade_id: u64, // Trade ID

    #[serde(rename = "p")]
    pub price: String, // Price

    #[serde(rename = "q")]
    pub quantity: String, // Quantity

    #[serde(rename = "b")]
    pub buyer_order_id: u64, // Buyer order ID

    #[serde(rename = "a")]
    pub seller_order_id: u64, // Seller order ID

    #[serde(rename = "T")]
    pub trade_time: u64, // Trade time

    #[serde(rename = "m")]
    pub is_buyer_market_maker: bool, // Is the buyer the market maker?

    #[serde(rename = "M")]
    pub ignore: bool, // Ignore
}
