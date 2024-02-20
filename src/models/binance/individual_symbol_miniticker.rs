use serde::{Serialize, Deserialize};

/// Individual Symbol Mini Ticker Stream
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndividualSymbolMiniTicker {
    #[serde(rename = "e")]
    pub event_type: String, // Event type

    #[serde(rename = "E")]
    pub event_time: u64, // Event time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "c")]
    pub close_price: String, // Close price

    #[serde(rename = "o")]
    pub open_price: String, // Open price

    #[serde(rename = "h")]
    pub high_price: String, // High price

    #[serde(rename = "l")]
    pub low_price: String, // Low price

    #[serde(rename = "v")]
    pub volume: String, // Total traded base asset volume

    #[serde(rename = "q")]
    pub quote_volume: String, // Total traded quote asset volume
}
