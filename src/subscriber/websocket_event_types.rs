use std::error::Error;
use serde_json::Value;
use crate::models::alchemy::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use crate::models::binance::aggregate_trade::AggregateTrade;
use crate::models::binance::all_market_minitickers::AllMarketMiniTickers;
use crate::models::binance::all_market_rolling_windows::AllMarketRollingWindows;
use crate::models::binance::all_market_tickers::AllMarketTickers;
use crate::models::binance::average_price::AveragePrice;
use crate::models::binance::binance_event_types::BinanceEventTypes;
use crate::models::binance::diff_depth::DiffDepth;
use crate::models::binance::individual_symbol_book_ticker::IndividualSymbolBookTicker;
use crate::models::binance::individual_symbol_miniticker::IndividualSymbolMiniTicker;
use crate::models::binance::individual_symbol_rolling_window::IndividualSymbolRollingWindow;
use crate::models::binance::individual_symbol_ticker::IndividualSymbolTicker;
use crate::models::binance::kline::Kline;
use crate::models::binance::partial_book_depth::PartialBookDepth;
use crate::models::binance::trade::Trade;
use crate::models::polygon::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon::polygon_crypto_trade_data::PolygonCryptoTradeData;
use crate::models::polygon::polygon_event_types::PolygonEventTypes;
use crate::util::serde_helper::deserialize_into;

///Trait used for event deserialization
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

    fn deserialize_event(value: &Value) -> Result<PolygonEventTypes, Box<dyn Error>> {
        match value["ev"].as_str() {
            Some("XQ") => {
                serde_json::from_value::<PolygonCryptoQuoteData>(value.clone())
                    .map(PolygonEventTypes::XqQuote)
                    .map_err(|e| e.into())
            },
            Some("XT") => {
                serde_json::from_value::<PolygonCryptoTradeData>(value.clone())
                    .map(PolygonEventTypes::XtTrade)
                    .map_err(|e| e.into())
            },
            Some("XL2") => {
                serde_json::from_value::<PolygonCryptoLevel2BookData>(value.clone())
                    .map(PolygonEventTypes::Xl2Level2book)
                    .map_err(|e| e.into())
            },
            Some("XA") => {
                serde_json::from_value::<PolygonCryptoAggregateData>(value.clone())
                    .map(PolygonEventTypes::XaAggregateMinute)
                    .map_err(|e| e.into())
            },
            Some("XAS") => {
                serde_json::from_value::<PolygonCryptoAggregateData>(value.clone())
                    .map(PolygonEventTypes::XasAggregateSecond)
                    .map_err(|e| e.into())
            },
            Some(unsupportedEvent) => Err(format!("Unsupported event type: {:?}", unsupportedEvent).into()),
            _ => Err("Unable to deserialize polygon websocket event".into()),

        }
    }

}

impl WebsocketEventTypes for BinanceEventTypes {
    fn event_type(&self) -> String {
        //todo
        match self {
            BinanceEventTypes::Trade(_) => "Trade".to_string(),
            BinanceEventTypes::AggregateTrade(_) => "AggregateTrade".to_string(),
            BinanceEventTypes::AllMarketMinitickers(_) => "AllMarketMinitickers".to_string(),
            BinanceEventTypes::AllMarketRollingWs(_) => "AllMarketRollingWs".to_string(),
            BinanceEventTypes::AllMarketTickers(_) => "AllMarketTickers".to_string(),
            BinanceEventTypes::DiffDepth(_) => "DiffDepth".to_string(),
            BinanceEventTypes::IndividualSymbolBookTicker(_) => "IndividualSymbolBookTicker".to_string(),
            BinanceEventTypes::IndividualSymbolMiniticker(_) => "IndividualSymbolMiniticker".to_string(),
            BinanceEventTypes::IndividualSymbolRollingW(_) => "IndividualSymbolRollingW".to_string(),
            BinanceEventTypes::IndividualSymbolTicker(_) => "IndividualSymbolTicker".to_string(),
            BinanceEventTypes::Kline(_) => "Kline".to_string(),
            BinanceEventTypes::PartialBookDepth(_) => "PartialBookDepth".to_string(),
            _ => {"Unsupported Binance event".to_string()}
        }
    }

    fn deserialize_event(value: &Value) -> Result<Self, Box<dyn Error>> {
        println!("{}", format!("Attempting to deserialize binance event:: {:?}", value));
        let result = match value["data"]["e"].as_str() {
            Some("trade") => deserialize_into::<Trade>(&value["data"]).map(BinanceEventTypes::Trade),
            Some("aggTrade") => deserialize_into::<AggregateTrade>(&value["data"]).map(BinanceEventTypes::AggregateTrade),
            Some("avgPrice") => deserialize_into::<AveragePrice>(&value["data"]).map(BinanceEventTypes::AveragePrice),
            Some("kline") => deserialize_into::<Kline>(&value["data"]).map(BinanceEventTypes::Kline),
            Some("24hrMiniTicker") => deserialize_into::<IndividualSymbolMiniTicker>(&value["data"]).map(BinanceEventTypes::IndividualSymbolMiniticker),
            Some("24hrTicker") => deserialize_into::<IndividualSymbolTicker>(&value["data"]).map(BinanceEventTypes::IndividualSymbolTicker),
            Some("allMarketMiniTickers") => deserialize_into::<AllMarketMiniTickers>(&value["data"]).map(BinanceEventTypes::AllMarketMinitickers),
            Some("allMarketTickers") => deserialize_into::<AllMarketTickers>(&value["data"]).map(BinanceEventTypes::AllMarketTickers),
            Some("1hTicker") | Some("4hTicker") | Some("1dTicker") =>
                deserialize_into::<IndividualSymbolRollingWindow>(&value["data"]).map(BinanceEventTypes::IndividualSymbolRollingW), //Window Sizes: 1h,4h,1d
            Some("allMarketRollingWindows") => deserialize_into::<AllMarketRollingWindows>(&value["data"]).map(BinanceEventTypes::AllMarketRollingWs),
            Some("depthUpdate") => deserialize_into::<DiffDepth>(&value["data"]).map(BinanceEventTypes::DiffDepth),
            Some(otherMessage) => {
                if let Ok(partial_book_depth) = deserialize_into::<PartialBookDepth>(&value["data"]) {
                    Ok(BinanceEventTypes::PartialBookDepth(partial_book_depth))
                } else if let Ok(individual_symbol_book_ticker) = deserialize_into::<IndividualSymbolBookTicker>(&value["data"]) {
                    Ok(BinanceEventTypes::IndividualSymbolBookTicker(individual_symbol_book_ticker))
                } else {
                    println!("Got other message: {:?}", otherMessage);
                    Err(format!("Deserialization error: unsupported type or malformed JSON :: {:?}", otherMessage).into())
                }
            },
            _ => {Err("Deserialization error: unsupported type or malformed JSON".into())}
        };

        match &result {
            Ok(deserialized) => println!("BINANCE EVENT: {:?}", deserialized),
            Err(e) => { } ,
        }

        result
    }
}