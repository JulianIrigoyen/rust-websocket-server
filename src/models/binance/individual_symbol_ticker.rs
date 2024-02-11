use serde::{Serialize, Deserialize};

/// Individual Symbol Ticker Stream Data Structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndividualSymbolTicker {
    #[serde(rename = "e")]
    pub event_type: String, // Event type

    #[serde(rename = "E")]
    pub event_time: u64, // Event time

    #[serde(rename = "s")]
    pub symbol: String, // Symbol

    #[serde(rename = "p")]
    pub price_change: String, // Price change

    #[serde(rename = "P")]
    pub price_change_percent: String, // Price change percent

    #[serde(rename = "w")]
    pub weighted_average_price: String, // Weighted average price

    #[serde(rename = "x")]
    pub first_trade_price: String, // First trade(F)-1 price

    #[serde(rename = "c")]
    pub last_price: String, // Last price

    #[serde(rename = "Q")]
    pub last_quantity: String, // Last quantity

    #[serde(rename = "b")]
    pub best_bid_price: String, // Best bid price

    #[serde(rename = "B")]
    pub best_bid_quantity: String, // Best bid quantity

    #[serde(rename = "a")]
    pub best_ask_price: String, // Best ask price

    #[serde(rename = "A")]
    pub best_ask_quantity: String, // Best ask quantity

    #[serde(rename = "o")]
    pub open_price: String, // Open price

    #[serde(rename = "h")]
    pub high_price: String, // High price

    #[serde(rename = "l")]
    pub low_price: String, // Low price

    #[serde(rename = "v")]
    pub total_traded_base_volume: String, // Total traded base asset volume

    #[serde(rename = "q")]
    pub total_traded_quote_volume: String, // Total traded quote asset volume

    #[serde(rename = "O")]
    pub statistics_open_time: u64, // Statistics open time

    #[serde(rename = "C")]
    pub statistics_close_time: u64, // Statistics close time

    #[serde(rename = "F")]
    pub first_trade_id: u64, // First trade ID

    #[serde(rename = "L")]
    pub last_trade_id: u64, // Last trade ID

    #[serde(rename = "n")]
    pub total_number_of_trades: u64, // Total number of trades
}
