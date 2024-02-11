use serde::{Serialize, Deserialize};

/// Kline/Candlestick Stream Data Structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Kline {
    #[serde(rename = "e")]
    pub event_type: String, // Event type

    #[serde(rename = "E")]
    pub event_time: u64, // Event time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "k")]
    pub kline: KlineDetail, // Kline details
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KlineDetail {
    #[serde(rename = "t")]
    pub start_time: u64, // Kline start time

    #[serde(rename = "T")]
    pub end_time: u64, // Kline end time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "i")]
    pub interval: String, // Interval

    #[serde(rename = "f")]
    pub first_trade_id: u64, // First trade ID

    #[serde(rename = "L")]
    pub last_trade_id: u64, // Last trade ID

    #[serde(rename = "o")]
    pub open_price: String, // Open price

    #[serde(rename = "c")]
    pub close_price: String, // Close price

    #[serde(rename = "h")]
    pub high_price: String, // High price

    #[serde(rename = "l")]
    pub low_price: String, // Low price

    #[serde(rename = "v")]
    pub volume: String, // Base asset volume

    #[serde(rename = "n")]
    pub number_of_trades: u64, // Number of trades

    #[serde(rename = "x")]
    pub is_closed: bool, // Is this kline closed?

    #[serde(rename = "q")]
    pub quote_volume: String, // Quote asset volume

    #[serde(rename = "V")]
    pub taker_buy_base_volume: String, // Taker buy base asset volume

    #[serde(rename = "Q")]
    pub taker_buy_quote_volume: String, // Taker buy quote asset volume

    #[serde(rename = "B")]
    pub ignore: String, // Ignore
}
