use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndividualSymbolBookTicker {
    #[serde(rename = "u")]
    pub update_id: u64, //  order book updateId

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "b")]
    pub best_bid_price: String,

    #[serde(rename = "B")]
    pub best_bid_qty: String,

    #[serde(rename = "a")]
    pub best_ask_price: String,

    #[serde(rename = "A")]
    pub best_ask_qty: String,
}
