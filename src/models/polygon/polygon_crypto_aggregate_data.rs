use serde::{Deserialize, Serialize};

/// Represents an aggregate data point for a specific cryptocurrency pair over a minute.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolygonCryptoAggregateData {
    /// The event type, indicating the type of data (e.g., "XA" for aggregates per minute).
    #[serde(rename = "ev")]
    pub event_type: String,

    /// The cryptocurrency pair (e.g., "BTC-USD").
    #[serde(rename = "pair")]
    pub pair: String,

    /// The open price for this aggregate window.
    #[serde(rename = "o")]
    pub open: f64,

    /// The close price for this aggregate window.
    #[serde(rename = "c")]
    pub close: f64,

    /// The high price for this aggregate window.
    #[serde(rename = "h")]
    pub high: f64,

    /// The low price for this aggregate window.
    #[serde(rename = "l")]
    pub low: f64,

    /// The volume of trades during this aggregate window.
    #[serde(rename = "v")]
    pub volume: f64,

    /// The start time for this aggregate window in Unix milliseconds.
    #[serde(rename = "s")]
    pub timestamp: i64,

    /// The end time for this aggregate window in Unix milliseconds.
    #[serde(rename = "e")]
    pub end_time: i64,

    /// The volume weighted average price.
    #[serde(rename = "vw")]
    pub vw: f64,

    /// The average trade size for this aggregate window.
    #[serde(rename = "z")]
    pub avg_trade_size: i64,
}
