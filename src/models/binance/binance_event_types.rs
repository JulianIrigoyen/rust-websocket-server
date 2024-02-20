use serde::{Serialize, Deserialize};

use crate::models::binance::{
    aggregate_trade, all_market_rolling_windows,all_market_tickers, all_market_minitickers,
    average_price,  diff_depth, individual_symbol_book_ticker, individual_symbol_ticker,
    individual_symbol_miniticker, individual_symbol_rolling_window, kline, partial_book_depth, trade
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BinanceEventTypes {
    AggregateTrade(aggregate_trade::AggregateTrade),
    AllMarketMinitickers(all_market_minitickers::AllMarketMiniTickers),
    AllMarketRollingWs(all_market_rolling_windows::AllMarketRollingWindows),
    AllMarketTickers(all_market_tickers::AllMarketTickers),
    DiffDepth(diff_depth::DiffDepth),
    IndividualSymbolBookTicker(individual_symbol_book_ticker::IndividualSymbolBookTickerStream),
    IndividualSymbolMiniticker(individual_symbol_miniticker::IndividualSymbolMiniTicker),
    IndividualSymbolRollingW(individual_symbol_rolling_window::IndividualSymbolRollingWindow),
    IndividualSymbolTicker(individual_symbol_ticker::IndividualSymbolTicker),
    Kline(kline::Kline),
    PartialBookDepth(partial_book_depth::PartialBookDepth),
    Trade(trade::Trade),
}
