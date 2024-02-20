use serde::{Serialize, Deserialize};

use crate::models::binance::{
    aggregate_trade::AggregateTrade,
    all_market_minitickers::AllMarketMiniTickers,
    all_market_rolling_windows::AllMarketRollingWindows,
    all_market_tickers::AllMarketTickers,
    average_price::AveragePrice,
    diff_depth::DiffDepth,
    individual_symbol_book_ticker::IndividualSymbolBookTicker,
    individual_symbol_miniticker::IndividualSymbolMiniTicker,
    individual_symbol_rolling_window::IndividualSymbolRollingWindow,
    individual_symbol_ticker::IndividualSymbolTicker,
    kline::Kline,
    partial_book_depth::PartialBookDepth,
    trade::Trade,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinanceEventTypes {
    Ping(),
    AggregateTrade(AggregateTrade),
    AllMarketMinitickers(AllMarketMiniTickers),
    AllMarketRollingWs(AllMarketRollingWindows),
    AllMarketTickers(AllMarketTickers),
    AveragePrice(AveragePrice),
    DiffDepth(DiffDepth),
    IndividualSymbolBookTicker(IndividualSymbolBookTicker),
    IndividualSymbolMiniticker(IndividualSymbolMiniTicker),
    IndividualSymbolRollingW(IndividualSymbolRollingWindow),
    IndividualSymbolTicker(IndividualSymbolTicker),
    Kline(Kline),
    PartialBookDepth(PartialBookDepth),
    Trade(Trade),
}
