use serde::{Serialize, Deserialize};
/**
    Partial Book Depth Streams: These streams provide updates to the top bids and asks for a symbol,
    which are crucial for traders who need to know the most competitive prices available for buying or selling without needing the entire order book.

    This data can be used to quickly make trading decisions based on the depth of the market.
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartialBookDepth {
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,

    #[serde(rename = "bids")]
    pub bids: Vec<(String, String)>, // Tuple of price and quantity

    #[serde(rename = "asks")]
    pub asks: Vec<(String, String)>, // Tuple of price and quantity
}
