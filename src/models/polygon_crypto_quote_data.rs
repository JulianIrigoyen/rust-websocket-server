use serde::{Deserialize, Serialize};

/// Represents a response from the Crypto Quote WebSocket.
#[derive(Serialize, Deserialize, Debug)]
pub struct PolygonCryptoQuoteData {
    /// The event type, indicating the type of data (e.g., "XQ" for crypto quotes).
    #[serde(rename = "ev")]
    pub event_type: String,

    /// The cryptocurrency pair (e.g., "BTC-USD").
    #[serde(rename = "pair")]
    pub pair: String,

    /// The bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,

    /// The bid size.
    #[serde(rename = "bs")]
    pub bid_size: f64,

    /// The ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,

    /// The ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,

    /// The timestamp in Unix milliseconds.
    #[serde(rename = "t")]
    pub timestamp: i64,

    /// The crypto exchange ID.
    #[serde(rename = "x")]
    pub exchange_id: i64,

    /// The timestamp that the tick was received by Polygon.
    #[serde(rename = "r")]
    pub received_timestamp: i64,
}
