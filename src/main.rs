// Required libraries:
// `crossbeam_channel` for efficient message passing between threads.
// `futures_util` for async stream processing.
// `timely` for dataflow-based stream processing.
// `tokio_tungstenite` for WebSocket communication.
// `url` for URL parsing.
// Custom model definitions for deserializing JSON messages.
#![allow(unused_variables)]
extern crate timely;

use crossbeam_channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};

use std::{env, thread};
use dotenv::dotenv;

use futures_util::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use timely::communication::allocator::{Generic, Thread};
use timely::dataflow::Scope;

use timely::dataflow::operators::{Exchange, Input, Inspect, Probe, ToStream, Filter, Broadcast, Partition};
use timely::dataflow::InputHandle;
use timely::execute_from_args;
use timely::worker::Worker;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;
use crate::models::polygon_event_types::PolygonEventTypes;
use crate::util::event_filters::{EventFilters, TradeSizeFilter, QuotePriceFilter, TradeFilter, FilterCriteria, ParameterizedTradeFilter, FilterValue};
mod models;
mod util;

/**

Rust Websocket Server :
   Establishes a WebSocket connection with Polygon.io's API, authenticates, and subscribes to various trading data streams.
    This includes trades, quotes, level 2 books, and aggregate data on a per-minute or per-second basis.
 */

/// Handles the connection and authentication process.
async fn connect(ws_url: Url, api_key: &str) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let (mut ws_stream, _) = connect_async(ws_url.clone())
        .await
        .expect("Failed to connect to WebSocket");
    println!("Connected to {}", ws_url);
    // Authenticate
    let auth_message = Message::Text(format!(r#"{{"action":"auth","params":"{}"}}"#, api_key));
    ws_stream
        .send(auth_message)
        .await
        .expect("Failed to authenticate");
    println!("Authenticated with {}", ws_url);
    ws_stream
}

/// The `async fn subscribe_with_parms` and `async fn subscribe_to_all_polygon_events` functions manage subscriptions to specific trading data channels.
async fn subscribe_with_parms(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    params: &[&str],
) {
    for &param in params {
        let sub = Message::Text(format!(r#"{{"action":"subscribe","params":"{}"}}"#, param));
        ws_stream
            .send(sub)
            .await
            .expect("Failed to send subscribe message");
        println!("Subscribed to {}", param);
    }
}

async fn subscribe_to_polygon_events(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    subscribe_aggregates_per_minute: bool,
    subscribe_aggregates_per_second: bool,
    subscribe_trades: bool,
    subscribe_quotes: bool,
    subscribe_level_2_books: bool
) {
    if subscribe_aggregates_per_minute {
        let aggregates_per_min_sub = Message::Text(r#"{"action":"subscribe","params":"XA.*"}"#.to_string());
        ws_stream.send(aggregates_per_min_sub).await.expect("Failed to send subscribe message for aggregates per minute");
        println!("Subscribed to all aggregates per minute");
    }

    if subscribe_aggregates_per_second {
        let aggregates_per_second_sub = Message::Text(r#"{"action":"subscribe","params":"XAS.*"}"#.to_string());
        ws_stream.send(aggregates_per_second_sub).await.expect("Failed to send subscribe message for aggregates per second");
        println!("Subscribed to all aggregates per second");
    }

    if subscribe_trades {
        let trades_sub = Message::Text(r#"{"action":"subscribe","params":"XT.*"}"#.to_string());
        ws_stream.send(trades_sub).await.expect("Failed to send subscribe message for all trades");
        println!("Subscribed to all trades");
    }

    if subscribe_quotes {
        let quotes_sub = Message::Text(r#"{"action":"subscribe","params":"XQ.*"}"#.to_string());
        ws_stream.send(quotes_sub).await.expect("Failed to send subscribe message for all quotes");
        println!("Subscribed to all quotes");
    }

    if subscribe_level_2_books {
        let level_2_books_sub = Message::Text(r#"{"action":"subscribe","params":"XL2.*"}"#.to_string());
        ws_stream.send(level_2_books_sub).await.expect("Failed to send subscribe message for all level 2 books");
        println!("Subscribed to all level 2 books");
    }

    // Add more conditions as needed for other types of subscriptions
}

/**
The `async fn process_websocket_messages` function listens for incoming WebSocket messages.
It deserializes these messages into strongly-typed Rust structs representing different kinds of trading events.
This deserialization is handled by the `fn deserialize_message` function, which checks each message against known data types and formats.
Upon successful deserialization, messages are sent to a Timely Dataflow computation for further processing.
 */
async fn process_websocket_messages(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: Sender<PolygonEventTypes>,
) {
    println!("Starting to process WebSocket messages");
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    let value: Result<serde_json::Value, _> = serde_json::from_str(&text);
                    match value {
                        Ok(val) => {
                            // Check if the message is an array and process each message individually
                            if val.is_array() {
                                for message in val.as_array().unwrap() {
                                    if let Some(event) = deserialize_message(message) {
                                        tx.send(event).expect("Failed to send event to dataflow");
                                    }
                                }
                            } else {
                                // Process a single message
                                if let Some(event) = deserialize_message(&val) {
                                    tx.send(event).expect("Failed to send event to dataflow");
                                }
                            }
                        }
                        Err(e) => eprintln!("Error deserializing message: {:?}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error receiving message: {:?}", e),
        }
    }
}

fn deserialize_message(value: &serde_json::Value) -> Option<PolygonEventTypes> {
    // Example for handling a Quote message works
    // if value["ev"] == "XQ" {
    //     if let Ok(quote_data) = serde_json::from_value::<PolygonCryptoQuoteData>(value.clone()) {
    //         println!("Deserialized a quote event: {}", quote_data.pair);
    //         return Some(PolygonEventTypes::XqQuote(quote_data));
    //     }
    // }

    match value["ev"].as_str() {
        Some("XQ") =>
            if let Ok(quote_data) = serde_json::from_value::<PolygonCryptoQuoteData>(value.clone()) {
                // println!("Deserialized a quote event: {}", quote_data.pair);
                return Some(PolygonEventTypes::XqQuote(quote_data));
            },
        Some("XT") =>
            if let Ok(trade_data) = serde_json::from_value::<PolygonCryptoTradeData>(value.clone()) {
                // println!("Deserialized a trade event: {}", trade_data.pair);
                return Some(PolygonEventTypes::XtTrade(trade_data.clone()));
            },
        Some("XL2") =>
            if let Ok(l2_book_data) = serde_json::from_value::<PolygonCryptoLevel2BookData>(value.clone()) {
                // println!("Deserialized a level 2 book event: {}", l2_book_data.pair);
                return Some(PolygonEventTypes::Xl2Level2book(l2_book_data.clone()));
            },
        Some("XA") =>
            if let Ok(min_data) = serde_json::from_value::<PolygonCryptoAggregateData>(value.clone()) {
                // println!("Deserialized a minute aggregate event: {}", min_data.pair);
                return Some(PolygonEventTypes::XaAggregateMinute(min_data.clone()));
            },
        Some("XAS") =>
            if let Ok(sec_data) = serde_json::from_value::<PolygonCryptoAggregateData>(value.clone()) {
                // println!("Deserialized a second aggregate event: {}", sec_data.pair);
                return Some(PolygonEventTypes::XasAggregateSecond(sec_data.clone()));
            },
        _ =>
            println!("Unknown event type: {}", value)
    }
    // Add handling for other types based on the "ev" field
    // ...
    None
}

fn run_dynamic_dataflow(event: PolygonEventTypes, worker: &mut timely::worker::Worker<timely::communication::Allocator>) {
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<PolygonEventTypes>();

        // stream.inspect(|x| println!("Data: {:?}", x));

        match event.clone() {
            PolygonEventTypes::XtTrade(trade_data) => {
                // Here, assume trade_data is an instance of PolygonEventTypes::XtTrade containing the actual trade data
                input_handle.send(event); // Send the event into the dataflow

                // Filter trades where size > 0.005 and inspect them
                stream
                    .filter(|x| {
                        //in this part, how can i invoke functions like large eth trades of more than 0.5 eth in size
                        //add this filter for the 10 most important cryypto
                        if let PolygonEventTypes::XtTrade(trade) = x {
                            trade.size > 0.005 // Check trade size
                        } else {
                            false
                        }
                    })
                    .inspect(|x| println!("Large BTC Trade: {:?}", x));
            },
            PolygonEventTypes::XaAggregateMinute(aggregate_data) => {
                input_handle.send(PolygonEventTypes::XaAggregateMinute(aggregate_data));
            },
            // Add other cases as needed
            _ => println!("Event type not supported for dynamic dataflow"),
        }

        // Make sure to advance the input handle to allow the dataflow to make progress
        input_handle.advance_to(1);
    });
}

fn filter_large_trades(trade: &PolygonCryptoTradeData) -> bool {
    trade.size > 0.005
}

fn filter_specific_pair(trade: &PolygonCryptoTradeData, pair: &str) -> bool {
    trade.pair == pair
}

fn run_dynamic_dataflowWithFilters(
    event: PolygonEventTypes,
    worker: &mut timely::worker::Worker<timely::communication::Allocator>,
    filters: EventFilters,
)
{
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<PolygonEventTypes>();

        // stream.inspect(|x| println!("Data: {:?}", x));

        input_handle.send(event); // Send the event into the dataflow

        // Apply all filters
        for filter in &filters.filters {
            let filter_clone = filter.clone();
            stream.filter(move |x| filter_clone.apply(x))
                .inspect(|x| println!("Filtered Event: {:?}", x));
        }

        input_handle.advance_to(1); // Make sure to advance the input handle
    });
}


/**
The main async function sets up the WebSocket connection, subscriptions, and spawns a separate thread for the Timely Dataflow computation.
 Inside the spawned thread, a Timely worker is initialized, and an input handle is created to feed data into the dataflow graph.
The dataflow computation is defined within the `worker.dataflow` closure.
Here, incoming events are simply logged using the `.inspect` operator for demonstration purposes.
 The `receiver.recv()` loop continuously reads deserialized events from a crossbeam channel, feeding them into the dataflow computation.
 */
#[tokio::main]
async fn main() {
    dotenv().ok(); // This loads the variables from .env into the environment

    let (sender, receiver) = bounded::<PolygonEventTypes>(5000); // Adjust buffer size as needed
    let polygon_ws_url = Url::parse("wss://socket.polygon.io/crypto").expect("Invalid WebSocket URL");
    let polygon_api_key = env::var("POLYGON_API_KEY").expect("Expected polygon_api_key to be set");
    println!("polygon_api_key: {}", polygon_api_key);

    let mut polygon_ws_stream = connect(polygon_ws_url, &polygon_api_key).await;

    subscribe_to_polygon_events(
        &mut polygon_ws_stream,
        true, // subscribe_aggregates_per_minute
        true, // subscribe_aggregates_per_second
        true, // subscribe_trades
        false, // subscribe_quotes
        false, // subscribe_level_2_books
    ).await;

    thread::spawn(move || {
        execute_from_args(std::env::args(), move |worker| {
            let mut input_handle: InputHandle<i64, PolygonEventTypes> = InputHandle::new();

            // Continuously read from the channel and process messages
            loop {
                match receiver.recv() {
                    Ok(event) => {
                        // Dynamically handle the event with a dataflow
                        //run_dynamic_dataflow(event, worker);

                        // Call run_dynamic_dataflow with the specified event and filters
                        let mut filters = EventFilters::new();
                        // let trade_size_filter = Arc::new(TradeSizeFilter {});
                        // let quote_price_filter = Arc::new(QuotePriceFilter {});

                        // Define criteria for both the trade size greater than 0.1 AND the "BTC-USD" pair
                        let btc_size_criteria = FilterCriteria {
                            field: "size".to_string(),
                            operation: ">".to_string(),
                            value: FilterValue::Number(0.5),
                        };

                        let btc_pair_criteria = FilterCriteria {
                            field: "pair".to_string(),
                            operation: "=".to_string(),
                            value: FilterValue::Text("BTC-USD".to_string()),
                        };


                        let trade_filter = Arc::new(TradeFilter {});

                        let param_filter = Arc::new(ParameterizedTradeFilter::new(vec![btc_size_criteria, btc_pair_criteria]));

                        // filters.add_filter(trade_size_filter);
                        // filters.add_filter(quote_price_filter);
                        // filters.add_filter(trade_filter);
                        filters.add_filter(param_filter);

                        run_dynamic_dataflowWithFilters(event, worker, filters);

                        worker.step(); // Drive the dataflow forward
                    }
                    Err(_) => {
                        break; // Exit loop if the sender is dropped/disconnected
                    }
                }
            }
        }).unwrap();
    });

    // // Add trade filters for BTC, ETH, and LINK
    // filters.add_trade_filter(|trade| trade.pair == "BTC-USD" && trade.size > 0.001);
    // filters.add_trade_filter(|trade| trade.pair == "ETH-USD" && trade.size > 0.5);
    // filters.add_trade_filter(|trade| trade.pair == "LINK-USD" && trade.size > 1000.0);



    process_websocket_messages(&mut polygon_ws_stream, sender).await;
}
