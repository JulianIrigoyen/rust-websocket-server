use std::sync::Arc;
use crate::models::polygon_event_types::PolygonEventTypes;
use serde::{Deserialize, Serialize};
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FilterValue {
    Number(f64),
    Text(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilterCriteria {
    pub field: String,
    pub operation: String,
    pub value: FilterValue,
}

pub struct ParameterizedTradeFilter {
    criteria: Vec<FilterCriteria>,
}

impl ParameterizedTradeFilter {
    pub fn new(criteria: Vec<FilterCriteria>) -> Self {
        ParameterizedTradeFilter { criteria }
    }
}

/// trait that can work generically with PolygonEventTypes
pub trait FilterFunction {
    fn apply(&self, event: &PolygonEventTypes) -> bool;
    // Example helper methods to extract field values and compare them
    fn extract_numeric_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<f64>;
    fn extract_string_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<String>;
    fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool;
    fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool;
}

impl FilterFunction for ParameterizedTradeFilter {
    fn apply(&self, event: &PolygonEventTypes) -> bool {
        if let PolygonEventTypes::XtTrade(trade) = event {
            for criterion in &self.criteria {
                match &criterion.value {
                    FilterValue::Number(num_val) => {
                        if let Some(field_value) = self.extract_numeric_field(&trade, &criterion.field) {
                            if !self.compare_numeric(field_value, &criterion.operation, num_val.clone()) {
                                return false;
                            }
                        }
                    },
                    FilterValue::Text(text_val) => {
                        if let Some(field_value) = self.extract_string_field(&trade, &criterion.field) {
                            if !self.compare_text(&field_value, &criterion.operation, text_val) {
                                return false;
                            }
                        }
                    },
                }
            }
            true
        } else {
            false
        }
    }

    // Example helper methods to extract field values and compare them
    fn extract_numeric_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<f64> {
        match field {
            "price" => Some(trade.clone().price),
            "size" => Some(trade.clone().size),
            _ => None,
        }
    }

    fn extract_string_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<String> {
        match field {
            "pair" => Some(trade.pair.clone()),
            _ => None,
        }
    }

    fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool {
        match operation {
            ">" => field_value > criterion_value,
            "<" => field_value < criterion_value,
            "=" => field_value == criterion_value,
            _ => false,
        }
    }

    fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool {
        match operation {
            "=" => field_value == criterion_value,
            "!=" => field_value != criterion_value,
            _ => false, // Other operations might not make sense for strings
        }
    }
}


/// Compares a trade's field value with a criterion value based on the operation.
fn compare(field_value: f64, operation: &str, criterion_value: f64) -> bool {
    match operation {
        ">" => field_value > criterion_value,
        "<" => field_value < criterion_value,
        "=" => field_value == criterion_value,
        _ => false,
    }
}

/// Implementations of FilterFunction trait for different specific filters.
/// Each implementation knows to handle its specific data type:
/// Example Filter to apply over XT Trade Events
pub struct TradeSizeFilter;
impl FilterFunction for TradeSizeFilter {
    fn apply(&self, event: &PolygonEventTypes) -> bool {
        match event {
            PolygonEventTypes::XtTrade(trade) => trade.size > 0.005,
            _ => false,
        }
    }

    fn extract_numeric_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<f64> {
        todo!()
    }

    fn extract_string_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<String> {
        todo!()
    }

    fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool {
        todo!()
    }

    fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool {
        todo!()
    }
}

/// Example Filter to apply over XQ Quote Events
pub struct QuotePriceFilter;
impl FilterFunction for QuotePriceFilter {
    fn apply(&self, event: &PolygonEventTypes) -> bool {
        match event {
            PolygonEventTypes::XqQuote(quote) => quote.bid_price > 100.0,
            _ => false,
        }
    }

    fn extract_numeric_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<f64> {
        todo!()
    }

    fn extract_string_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<String> {
        todo!()
    }

    fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool {
        todo!()
    }

    fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool {
        todo!()
    }
}

pub struct TradeFilter;
impl FilterFunction for TradeFilter {
    fn apply(&self, event: &PolygonEventTypes) -> bool {
        match event {
            PolygonEventTypes::XtTrade(trade) => {
                match trade.clone().pair.as_str() {
                    "BTC-USD" => trade.size  > 0.1, // Assuming size is in BTC and 1 BTC = 44k USD
                    "ETH-USD" => trade.size > 10.0, // Assuming size is in ETH
                    "SOL-USD" => trade.size > 100.0, // Assuming size is in SOL
                    _ => false,
                }
            },
            _ => false,
        }
    }

    fn extract_numeric_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<f64> {
        todo!()
    }

    fn extract_string_field(&self, trade: &PolygonCryptoTradeData, field: &str) -> Option<String> {
        todo!()
    }

    fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool {
        todo!()
    }

    fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool {
        todo!()
    }
}



pub struct EventFilters {
    pub(crate) filters: Vec<Arc<dyn FilterFunction>>,
}

impl EventFilters {
    pub fn new() -> Self {
        Self { filters: Vec::new() }
    }

    pub fn add_filter(&mut self, filter: Arc<dyn FilterFunction>) {
        self.filters.push(filter);
    }
}