use std::error::Error;
use serde_json::Value;
use crate::models::alchemy::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use crate::models::binance::binance_event_types::BinanceEventTypes;
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

impl WebsocketEventTypes for BinanceEventTypes {
    fn event_type(&self) -> String {
        //todo
        match self {
            BinanceEventTypes::Trade(_) => "Trade".to_string(),
            BinanceEventTypes::AggregateTrade(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::AllMarketMinitickers(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::AllMarketRollingWs(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::AllMarketTickers(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::DiffDepth(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::IndividualSymbolBookTicker(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::IndividualSymbolMiniticker(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::IndividualSymbolRollingW(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::IndividualSymbolTicker(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::Kline(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::PartialBookDepth(_) => "AggregateTrade".to_string(),
        }
    }

    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>> {
        serde_json::from_value(value.clone()).map_err(Into::into)
    }
}