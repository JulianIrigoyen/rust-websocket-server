use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_event_types::PolygonEventTypes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Defines a tracker for the price movements of cryptocurrency pairs
pub struct PolygonPriceActionTracker {
    // A thread-safe (Arc + Mutex) HashMap to store the last prices of cryptocurrency pairs
    // The key is the pair as a String (e.g., "BTC-USD"), and the value is the last price as f64
    last_prices: Arc<Mutex<HashMap<String, f64>>>,
    spike_threshold_percent: f64, // field for spike/drop detection threshold
}

impl PolygonPriceActionTracker {
    pub(crate) fn new(spike_threshold_percent: f64) -> Self {
        PolygonPriceActionTracker {
            last_prices: Arc::new(Mutex::new(HashMap::new())),
            spike_threshold_percent,
        }
    }

    fn process(&self, aggregate: &PolygonCryptoAggregateData) {
        // Lock the HashMap for safe concurrent access, unwrapping the result to handle potential errors
        let mut last_prices = self.last_prices.lock().unwrap();
        // Retrieves or inserts the opening price as the last price for the given cryptocurrency pair
        let last_price = last_prices
            .entry(aggregate.pair.clone())
            .or_insert(aggregate.open);

        // Compare the closing price with the last price to determine the price movement direction
        if aggregate.close > *last_price {
            println!(
                "{} price went up from last tick. Last: {}, Current: {}",
                aggregate.pair, last_price, aggregate.close
            );
        } else if aggregate.close < *last_price {
            println!(
                "{} price went down from last tick. Last: {}, Current: {}",
                aggregate.pair, last_price, aggregate.close
            );
        } else {
            println!(
                "{} price stayed the same as the last tick. Price: {}",
                aggregate.pair, aggregate.close
            );
        }

        let price_change_percent = ((aggregate.close - *last_price) / *last_price) * 100.0;

        if price_change_percent.abs() > self.spike_threshold_percent {
            println!(
                "{} experienced a {} of {:.2}% from the last tick. Last: {}, Current: {}",
                aggregate.pair,
                if price_change_percent > 0.0 {
                    "spike"
                } else {
                    "drop"
                },
                price_change_percent,
                last_price,
                aggregate.close
            );
        }

        // Update the last seen price for the pair
        *last_price = aggregate.close;
    }
}

impl PolygonPriceActionTracker {
    pub(crate) fn apply(&self, event: &PolygonEventTypes) {
        match event {
            PolygonEventTypes::XaAggregateMinute(aggregate) => self.process(aggregate),
            PolygonEventTypes::XasAggregateSecond(aggregate) => self.process(aggregate),
            _ => {} // This tracker only processes aggregate data
        }
    }
}
