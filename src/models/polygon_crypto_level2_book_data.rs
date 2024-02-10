use serde::{Deserialize, Serialize};

/// Represents a response from the Level 2 WebSocket.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolygonCryptoLevel2BookData {
    /// The event type, indicating the type of data (e.g., "XL2" for Level 2 data).
    #[serde(rename = "ev")]
    pub event_type: String,

    /// The cryptocurrency pair (e.g., "BTC-USD").
    #[serde(rename = "pair")]
    pub pair: String,

    /// The timestamp in Unix milliseconds.
    #[serde(rename = "t")]
    pub timestamp: i64,

    /// The timestamp that the tick was received by Polygon.
    #[serde(rename = "r")]
    pub received_timestamp: i64,

    /// The crypto exchange ID.
    #[serde(rename = "x")]
    pub exchange_id: i64,

    /// An array of bid prices with a maximum depth of 100.
    #[serde(rename = "b")]
    pub bid_prices: Vec<Vec<f64>>,

    /// An array of ask prices with a maximum depth of 100.
    #[serde(rename = "a")]
    pub ask_prices: Vec<Vec<f64>>,
}

impl Default for PolygonCryptoLevel2BookData {
    fn default() -> Self {
        Self {
            event_type: "".to_string(),
            pair: "".to_string(),
            timestamp: 0,
            received_timestamp: 0,
            exchange_id: 0,
            bid_prices: Vec::new(),
            ask_prices: Vec::new(),
        }
    }
}
