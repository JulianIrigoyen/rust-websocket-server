use serde::{Serialize, Deserialize};
use crate::models::binance::individual_symbol_rolling_window::IndividualSymbolRollingWindow;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AllMarketRollingWindows {
    #[serde(flatten)]
    pub statistics: Vec<IndividualSymbolRollingWindow>, // Utilizing the IndividualSymbolRollingWindowStatisticsStream struct
}
