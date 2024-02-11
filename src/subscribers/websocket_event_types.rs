use std::error::Error;
use serde_json::Value;
use crate::models::alchemy::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use crate::models::polygon::polygon_event_types::PolygonEventTypes;

pub trait WebsocketEventTypes: Sized {
    // Returns a descriptive name or type of the event.
    fn event_type(&self) -> String;

    // Method to deserialize a JSON value into a specific event type.
    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>>;
}


impl WebsocketEventTypes for AlchemyEventTypes {
    fn event_type(&self) -> String {
        match self {
            AlchemyEventTypes::AlchemyMinedTransactions(_) => "AlchemyMinedTransactions".to_string(),
        }
    }

    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>> {
        serde_json::from_value::<AlchemyMinedTransactionData>(value.clone())
            .map(AlchemyEventTypes::AlchemyMinedTransactions)
            .map_err(|e| e.into())
    }
}

impl WebsocketEventTypes for PolygonEventTypes {
    fn event_type(&self) -> String {
        match self {
            PolygonEventTypes::XaAggregateMinute(_) => "XaAggregateMinute".to_string(),
            PolygonEventTypes::XasAggregateSecond(_) => "XasAggregateSecond".to_string(),
            PolygonEventTypes::XtTrade(_) => "XtTrade".to_string(),
            PolygonEventTypes::XqQuote(_) => "XqQuote".to_string(),
            PolygonEventTypes::Xl2Level2book(_) => "Xl2Level2book".to_string(),
        }
    }

    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>> {
        serde_json::from_value(value.clone()).map_err(Into::into)
    }
}