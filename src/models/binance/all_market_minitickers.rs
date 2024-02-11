use serde::{Serialize, Deserialize};
use crate::models::binance::individual_symbol_miniticker::IndividualSymbolMiniTicker;

/// All Market Mini Tickers Stream Data Structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllMarketMiniTickers{
    #[serde(flatten)]
    pub tickers: Vec<IndividualSymbolMiniTicker>
}
