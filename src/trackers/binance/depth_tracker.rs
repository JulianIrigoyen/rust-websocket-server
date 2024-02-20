use std::collections::HashMap;
use crate::models::binance::binance_event_types::BinanceEventTypes;
use crate::models::binance::diff_depth::DiffDepth;
use crate::models::binance::kline::Kline;
use crate::models::binance::partial_book_depth::PartialBookDepth;
use crate::models::binance::trade::Trade;

/**

Responsibility: Monitor and analyze the order book depth data (bids and asks) for various symbols.
Functionality:  Maintain a current state of the order book for each symbol, highlighting potential support and resistance levels based on the volume of bids and asks.


Analysis:

    Market Depth: Market depth refers to the volume of orders waiting to be executed at different price levels for a particular asset. It's visualized in what's often called an "order book". Depth data shows the demand (bids) and supply (asks) at different price points and the volume available at each level.
        * Bids: Orders from buyers to purchase the asset at a certain price. They are listed in descending order with the highest bid at the top.
        * Asks: Orders from sellers to sell the asset at a certain price. They are listed in ascending order with the lowest ask at the top.

Depth data is crucial because it provides insight into potential resistance (in the case of asks) and support (in the case of bids) levels.
- High volume at a bid level suggests strong buying interest that could act as support
- High volume at an ask level indicates selling interest that could act as resistance.

 */

struct DepthTracker {
}

impl DepthTracker {
    pub fn new(period: usize) -> Self {
        Self {

            //todo what should an efficient tracker be initialized with? we will be processing a lot of symbols so probably a mutable map for the state of each?
        }
    }

    pub(crate) fn apply(&mut self, event: &BinanceEventTypes) {
        match event {
            BinanceEventTypes::Trade(data) => self.process_binance_trade(data),
            BinanceEventTypes::PartialBookDepth(data) => self.update_partial_depth_data(data),
            BinanceEventTypes::DiffDepth(data) => self.update_diff_depth_data(data),
            _ => {}
        }
    }

    pub(crate) fn process_binance_trade(&mut self, trade: &Trade) {
        // Update price changes based on the trade price
        // Recalculate RSI for relevant intervals
    }

    // Example method stub for processing depth data
    pub(crate) fn update_partial_depth_data(&mut self, depth: &PartialBookDepth) {
        // Update internal market depth state
    }

    pub(crate) fn update_diff_depth_data(&mut self, depth: &DiffDepth) {
        // Update internal market depth state
    }
}
