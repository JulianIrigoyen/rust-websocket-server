use serde::{Serialize, Deserialize};
use crate::models::binance::individual_symbol_ticker::IndividualSymbolTicker;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllMarketTickers{
    #[serde(flatten)]
    pub tickers: Vec<IndividualSymbolTicker>
}
