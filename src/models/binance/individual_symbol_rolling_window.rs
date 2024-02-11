use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndividualSymbolRollingWindow {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: u64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "p")]
    pub price_change: String,

    #[serde(rename = "P")]
    pub price_change_percent: String,

    #[serde(rename = "o")]
    pub open_price: String,

    #[serde(rename = "h")]
    pub high_price: String,

    #[serde(rename = "l")]
    pub low_price: String,

    #[serde(rename = "c")]
    pub last_price: String,

    #[serde(rename = "w")]
    pub weighted_average_price: String,

    #[serde(rename = "v")]
    pub total_traded_base_volume: String,

    #[serde(rename = "q")]
    pub total_traded_quote_volume: String,

    #[serde(rename = "O")]
    pub statistics_open_time: u64,

    #[serde(rename = "C")]
    pub statistics_close_time: u64,

    #[serde(rename = "F")]
    pub first_trade_id: u64,

    #[serde(rename = "L")]
    pub last_trade_id: u64,

    #[serde(rename = "n")]
    pub total_number_of_trades: u64,
}
