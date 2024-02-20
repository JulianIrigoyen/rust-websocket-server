use serde::{Serialize, Deserialize};
/**
    Diff. Depth Stream: The differential depth stream provides updates on changes to the order book for a symbol.

    This stream is essential for maintaining a local copy of the order book. It allows traders to see how the market depth changes in real-time,
        offering insights into potential price movements based on buying and selling pressure.

    This can be particularly useful for algorithmic trading strategies that require accurate and up-to-date order book information to execute trades effectively.
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiffDepth {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "U")]
    pub first_update_id_in_event: u64,

    #[serde(rename = "u")]
    pub final_update_id_in_event: u64,

    #[serde(rename = "b")]
    pub bids: Vec<(String, String)>, // Tuple of price and quantity

    #[serde(rename = "a")]
    pub asks: Vec<(String, String)>, // Tuple of price and quantity
}
