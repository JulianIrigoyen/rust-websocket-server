use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;
use crate::models::polygon_event_types::PolygonEventTypes;

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

// This struct now holds a map from currency pairs to their respective criteria.
pub struct ParameterizedTradeFilter {
    criteria_by_pair: HashMap<String, Vec<FilterCriteria>>,
}

impl ParameterizedTradeFilter {
    pub fn new(criteria_by_pair: HashMap<String, Vec<FilterCriteria>>) -> Self {
        ParameterizedTradeFilter { criteria_by_pair }
    }
}

pub trait FilterFunction {
    fn apply(&self, event: &PolygonEventTypes) -> bool;
}

impl FilterFunction for ParameterizedTradeFilter {
    fn apply(&self, event: &PolygonEventTypes) -> bool {
        match event {
            PolygonEventTypes::XtTrade(trade) => {
                if let Some(criteria) = self.criteria_by_pair.get(&trade.pair) {
                    criteria
                        .iter()
                        .all(|criterion| self.meets_criterion(trade, criterion))
                } else {
                    true // If no criteria for the pair, pass the trade through
                }
            }
            _ => false,
        }
    }
}

impl ParameterizedTradeFilter {
    fn meets_criterion(&self, trade: &PolygonCryptoTradeData, criterion: &FilterCriteria) -> bool {
        match &criterion.value {
            FilterValue::Number(num_val) => match criterion.field.as_str() {
                "price" => {
                    self.compare_numeric(trade.clone().price, &criterion.operation, num_val.clone())
                }
                "size" => {
                    self.compare_numeric(trade.clone().size, &criterion.operation, num_val.clone())
                }
                _ => false,
            },
            FilterValue::Text(text_val) => match criterion.field.as_str() {
                "pair" => self.compare_text(&trade.pair, &criterion.operation, text_val),
                _ => false,
            },
        }
    }

    fn compare_numeric(&self, field_value: f64, operation: &str, criterion_value: f64) -> bool {
        match operation {
            ">" => field_value > criterion_value,
            "<" => field_value < criterion_value,
            "=" => (field_value - criterion_value).abs() < f64::EPSILON,
            _ => false,
        }
    }

    fn compare_text(&self, field_value: &str, operation: &str, criterion_value: &str) -> bool {
        match operation {
            "=" => field_value == criterion_value,
            "!=" => field_value != criterion_value,
            _ => false,
        }
    }
}

pub struct EventFilters {
    pub(crate) filters: Vec<Arc<dyn FilterFunction>>,
}

impl EventFilters {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    pub fn add_filter(&mut self, filter: Arc<dyn FilterFunction>) {
        self.filters.push(filter);
    }
}
