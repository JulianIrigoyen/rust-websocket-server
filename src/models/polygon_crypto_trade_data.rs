use serde::{Deserialize, Serialize};

/// Represents a response from the Crypto Trade WebSocket.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolygonCryptoTradeData {
    /// The event type, indicating the type of data (e.g., "XT" for crypto trades).
    #[serde(rename = "ev")]
    pub event_type: String,

    /// The cryptocurrency pair (e.g., "BTC-USD").
    #[serde(rename = "pair")]
    pub pair: String,

    /// The price of the trade.
    #[serde(rename = "p")]
    pub price: f64,

    /// The timestamp in Unix milliseconds.
    #[serde(rename = "t")]
    pub timestamp: i64,

    /// The size of the trade.
    #[serde(rename = "s")]
    pub size: f64,

    /// The conditions of the trade.
    #[serde(rename = "c")]
    pub conditions: Vec<i64>,

    /// The ID of the trade (optional).
    #[serde(rename = "i")]
    pub trade_id: Option<String>,

    /// The crypto exchange ID.
    #[serde(rename = "x")]
    pub exchange_id: i64,

    /// The timestamp that the tick was received by Polygon.
    #[serde(rename = "r")]
    pub received_timestamp: i64,
}

impl Default for PolygonCryptoTradeData {
    fn default() -> Self {
        PolygonCryptoTradeData {
            // initialize fields with default values
            event_type: "".to_string(),
            pair: "".to_string(),
            price: 0.0,
            timestamp: 0,
            size: 0.0,
            conditions: vec![],
            trade_id: None,
            exchange_id: 0,
            received_timestamp: 0,
        }
    }
}
