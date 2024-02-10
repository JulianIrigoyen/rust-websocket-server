// Required libraries:
// `crossbeam_channel` for efficient message passing between threads.
// `futures_util` for async stream processing.
// `timely` for dataflow-based stream processing.
// `tokio_tungstenite` for WebSocket communication.
// `url` for URL parsing.
// Custom model definitions for deserializing JSON messages.
#![allow(unused_variables)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate dotenv_codegen;
extern crate timely;

use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
    types::Address,
};

use std::{env, thread};
use std::collections::HashMap;
use std::sync::Arc;

use crossbeam_channel::{bounded, Sender};
use diesel::prelude::*;
use dotenv::dotenv;
use ethers::prelude::{Middleware, U64};
use futures_util::{sink::SinkExt, stream::StreamExt};
use timely::dataflow::InputHandle;
use timely::dataflow::operators::{Filter, Input, Inspect};
use timely::execute_from_args;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream,
};
use url::Url;

use crate::db::db_session_manager::DbSessionManager;
use crate::models::alchemy_event_types::AlchemyEventTypes;
use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;
use crate::models::polygon_event_types::PolygonEventTypes;
use crate::server::ws_server;
use crate::subscriber::alchemy_subscriber::AlchemySubscriber;
use crate::trackers::price_movement_tracker::PriceActionTracker;
use crate::util::event_filters::{
    EventFilters, FilterCriteria, FilterValue, ParameterizedFilter,
};

mod models;
mod server;
mod trackers;
mod util;
mod db;
mod schema;
mod web3;
mod subscriber;

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
async fn subscribe_to_polygon_events_with_params(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    params: &[&str],
) {
    for &param in params {
        let sub = Message::Text(format!(r#"{{"action":"subscribe","params":"{}"}}"#, param));
        ws_stream
            .send(sub)
            .await
            .expect("Failed to send subscribe message for all trades");

        println!("Subscribed to {}", param);
    }
}

async fn subscribe_to_binance_events(
    binance_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    subscribe_agg_trades: bool,
    subscribe_trades: bool,
    subscribe_klines: bool,
) {
    if subscribe_agg_trades {
        let agg_trade_sub = Message::Text(r#"{"method":"SUBSCRIBE","params":["btcusdt@aggTrade"],"id":1}"#.to_string());
        binance_stream
            .send(agg_trade_sub)
            .await
            .expect("Failed to send subscribe message for aggregate trades");
        println!("Subscribed to aggregate trades");
    }

    if subscribe_trades {
        let trades_sub = Message::Text(r#"{"method":"SUBSCRIBE","params":["btcusdt@trade"],"id":2}"#.to_string());
        binance_stream
            .send(trades_sub)
            .await
            .expect("Failed to send subscribe message for trades");
        println!("Subscribed to trades");
    }

    if subscribe_klines {
        let klines_sub = Message::Text(r#"{"method":"SUBSCRIBE","params":["btcusdt@kline_1m"],"id":3}"#.to_string());
        binance_stream
            .send(klines_sub)
            .await
            .expect("Failed to send subscribe message for Kline data");
        println!("Subscribed to Kline data");
    }
}

/// Utility function to easily subscribe to polygon events.
async fn subscribe_to_polygon_events(
    polygon_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    subscribe_aggregates_per_minute: bool,
    subscribe_aggregates_per_second: bool,
    subscribe_trades: bool,
    subscribe_quotes: bool,
    subscribe_level_2_books: bool,
) {
    if subscribe_aggregates_per_minute {
        let aggregates_per_min_sub =
            Message::Text(r#"{"action":"subscribe","params":"XA.*"}"#.to_string());
        polygon_stream
            .send(aggregates_per_min_sub)
            .await
            .expect("Failed to send subscribe message for aggregates per minute");
        println!("Subscribed to all aggregates per minute");
    }

    if subscribe_aggregates_per_second {
        let aggregates_per_second_sub =
            Message::Text(r#"{"action":"subscribe","params":"XAS.*"}"#.to_string());
        polygon_stream
            .send(aggregates_per_second_sub)
            .await
            .expect("Failed to send subscribe message for aggregates per second");
        println!("Subscribed to all aggregates per second");
    }

    if subscribe_trades {
        let trades_sub = Message::Text(r#"{"action":"subscribe","params":"XT.*"}"#.to_string());
        polygon_stream
            .send(trades_sub)
            .await
            .expect("Failed to send subscribe message for all trades");
        println!("Subscribed to polygon all trades");
    }

    if subscribe_quotes {
        let quotes_sub = Message::Text(r#"{"action":"subscribe","params":"XQ.*"}"#.to_string());
        polygon_stream
            .send(quotes_sub)
            .await
            .expect("Failed to send subscribe message for all quotes");
        println!("Subscribed to all quotes");
    }

    if subscribe_level_2_books {
        let level_2_books_sub =
            Message::Text(r#"{"action":"subscribe","params":"XL2.*"}"#.to_string());
        polygon_stream
            .send(level_2_books_sub)
            .await
            .expect("Failed to send subscribe message for all level 2 books");
        println!("Subscribed to all level 2 books");
    }
}

/**
The `async fn process_websocket_messages` function listens for incoming WebSocket messages.
It deserializes these messages into strongly-typed Rust structs representing different kinds of trading events.
This deserialization is handled by the `fn deserialize_message` function, which checks each message against known data types and formats.
Upon successful deserialization, messages are sent to a Timely Dataflow computation for further processing.
 */
async fn consume_polygon_stream(
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
    match value["ev"].as_str() {
        Some("XQ") => {
            if let Ok(quote_data) = serde_json::from_value::<PolygonCryptoQuoteData>(value.clone())
            {
                // println!("Deserialized a quote event: {}", quote_data.pair);
                return Some(PolygonEventTypes::XqQuote(quote_data));
            }
        }
        Some("XT") => {
            if let Ok(trade_data) = serde_json::from_value::<PolygonCryptoTradeData>(value.clone())
            {
                // println!("Deserialized a trade event: {}", trade_data.pair);
                return Some(PolygonEventTypes::XtTrade(trade_data.clone()));
            }
        }
        Some("XL2") => {
            if let Ok(l2_book_data) =
                serde_json::from_value::<PolygonCryptoLevel2BookData>(value.clone())
            {
                // println!("Deserialized a level 2 book event: {}", l2_book_data.pair);
                return Some(PolygonEventTypes::Xl2Level2book(l2_book_data.clone()));
            }
        }
        Some("XA") => {
            if let Ok(min_data) =
                serde_json::from_value::<PolygonCryptoAggregateData>(value.clone())
            {
                // println!("Deserialized a minute aggregate event: {}", min_data.pair);
                return Some(PolygonEventTypes::XaAggregateMinute(min_data.clone()));
            }
        }
        Some("XAS") => {
            if let Ok(sec_data) =
                serde_json::from_value::<PolygonCryptoAggregateData>(value.clone())
            {
                // println!("Deserialized a second aggregate event: {}", sec_data.pair);
                return Some(PolygonEventTypes::XasAggregateSecond(sec_data.clone()));
            }
        }
        _ => println!("Unknown event type: {}", value),
    }
    // ...
    None
}

// Example - how to run a dataflow with a simple filter. This function is given superpowers in its younger brother below
fn run_dynamic_dataflow(
    event: PolygonEventTypes,
    worker: &mut timely::worker::Worker<timely::communication::Allocator>,
) {
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<PolygonEventTypes>();

        // stream.inspect(|x| println!("[[MAIN DATAFLOW]]: Got message {:?}", x));

        match event.clone() {
            PolygonEventTypes::XtTrade(trade_data) => {
                //handle XT Trade data
                input_handle.send(event);
                // Filter trades where size > 0.005 and inspect them
                stream
                    .filter(|x| {
                        //in this part, how can i invoke functions like large eth trades of more than 0.5 eth in size
                        //add this filter for the 10 most important crypto
                        if let PolygonEventTypes::XtTrade(trade) = x {
                            trade.size > 0.005 // Check trade size
                        } else {
                            false
                        }
                    })
                    .inspect(|x| println!("Large BTC Trade: {:?}", x));
            }
            PolygonEventTypes::XaAggregateMinute(aggregate_data) => {
                input_handle.send(PolygonEventTypes::XaAggregateMinute(aggregate_data));
            }
            // Add other cases as needed
            _ => println!("Event type not supported for dynamic dataflow"),
        }

        // Make sure to advance the input handle to allow the dataflow to make progress
        input_handle.advance_to(1);
    });
}

fn run_filtered_polygon_dataflow(
    event: PolygonEventTypes,
    worker: &mut timely::worker::Worker<timely::communication::Allocator>,
    filters: EventFilters,
    db_session_manager: Arc<DbSessionManager>,
) {
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<PolygonEventTypes>();

        // stream.inspect(|x| println!("Data: {:?}", x));

        // Create an input handle and stream for PolygonEventTypes
        input_handle.send(event); // Send the event into the dataflow
        // Create a new instance of the price movement tracker
        let price_movement_tracker = PriceActionTracker::new(0.0001);
        // Inspect each event and apply the tracker's logic
        stream.inspect(move |event| {
            price_movement_tracker.apply(event);
        });

        // Apply all filters
        for filter in &filters.filters {
            let filter_clone = filter.clone();
            let db_session_manager_cloned = db_session_manager.clone(); // Clone for use in the closure

            stream
                .filter(move |x| filter_clone.apply(x))
                .inspect(move |x| { // Use `move` to capture variables by value
                    println!("Filtered Polygon Event: {:?}", x);

                    match db_session_manager_cloned.persist_event(x) {
                        Ok(_) => println!("Polygon Event successfully persisted."),
                        Err(e) => eprintln!("Error persisting polygon event: {:?}", e),
                    }
                });
        }

        input_handle.advance_to(1); // Make sure to advance the input handle
    });
}

/**
The IUniswapV2Pair is a struct that is generated from the abigen!() macro.
    The IUniswapV2Pair::new() function is used to create a new instance of the contract, taking in an Address and an Arc<M> as arguments, where M is any type that implements the Middleware trait.

    Note that the provider is wrapped in an Arc when being passed into the new() function.
        It is very common to wrap a provider in an Arc to share the provider across threads.
 */
abigen!(
    IUniswapV2Pair,
    "[function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)]"
);

/**
The main async function sets up the WebSocket connection, subscriptions, and spawns a separate thread for the Timely Dataflow computation.
 Inside the spawned thread, a Timely worker is initialized, and an input handle is created to feed data into the dataflow graph.
The dataflow computation is defined within the `worker.dataflow` closure.

 The `receiver.recv()` loop continuously reads deserialized events from a crossbeam channel,
feeding them into the dataflow computation.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///Load env vars and connect to DB
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_session_manager = Arc::new(DbSessionManager::new(&database_url));

    // Attempt to fetch a database connection
    match db_session_manager.get_connection() {
        Ok(_) => println!("Successfully connected to the database."),
        Err(e) => eprintln!("Failed to connect to the database: {:?}", e),
    }

    /// Websocket Server Initialization
    let ws_host = env::var("WS_SERVER_HOST").expect("WS_HOST must be set");
    let ws_port = env::var("WS_SERVER_PORT").expect("WS_PORT must be set");

    // This task runs the WebSocket server for broadcasting processed results
    let ws_server_task = tokio::spawn(async move {
        let ws_server = ws_server::WebSocketServer::new(ws_host, ws_port);
        ws_server.run().await
    });

    /// Web3 Ethereum Node Alchemy Provider Initialization
    let alchemy_http_url = env::var("ALCHEMY_HTTP_URL").expect("ALCHEMY_HTTP_URL must be set");
    let alchemy_ws_url = env::var("ALCHEMY_WS_URL").expect("ALCHEMY_WS_URL must be set");
    let alchemy_api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set");

    let (alchemy_event_sender, alchemy_event_receiver) = bounded::<AlchemyEventTypes>(5000);
    let alchemy_ws_message_processing_task = tokio::spawn(async move {
        let alchemy_provider = Arc::new(Provider::try_from(format!("{}{}", alchemy_http_url, alchemy_api_key)));
        let mut alchemy_subscriber = AlchemySubscriber::new(alchemy_ws_url, alchemy_api_key);
        let mut alchemy_ws_stream = alchemy_subscriber.connect().await;

        alchemy_subscriber.subscribe(&mut alchemy_ws_stream, "alchemy_minedTransactions").await;

        while let Some(message) = alchemy_ws_stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    println!("Received message: {}", text);
                    // Deserialize and process the message as needed
                }
                Ok(_) => {} // Handle other message types as needed
                Err(e) => eprintln!("Error receiving message: {:?}", e),
            }
        }
    });

    let alchemy_dataflow_task = tokio::spawn(async move {
        // Initialize and execute the dataflow
        thread::spawn(move || {
            execute_from_args(env::args(), move |worker| {
                let input_handle: InputHandle<i64, AlchemyEventTypes> = InputHandle::new();

                // Continuously read from the channel and process messages
                loop {
                    // Consume the crossbeam channel with messages from the websocket
                    match alchemy_event_receiver.recv() {
                        Ok(event) => {

                            worker.step(); // Drive the dataflow forward
                        }
                        Err(_) => {
                            break; // Exit loop if the sender is dropped/disconnected
                        }
                    }
                }
            })
                .unwrap();
        });
    });
    /// PolygonIO Websocket setup
    //Create channel for events to be sent from the websocket to the dataflow
    let polygon_ws_url_string = env::var("POLYGON_WS_URL").expect("Expected POLYGON_API_KEY to be set");
    let polygon_ws_url = Url::parse(&*polygon_ws_url_string).expect("Invalid WebSocket URL");
    let polygon_api_key = env::var("POLYGON_API_KEY").expect("Expected POLYGON_API_KEY to be set");

    let (polygon_event_sender, polygon_event_receiver) = bounded::<PolygonEventTypes>(5000);
    // This task handles incoming WebSocket messages from the Polygon.io API
    let polygon_ws_message_processing_task = tokio::spawn(async move {
        // Subscribe to Polygon.io channels here
        let mut polygon_ws_stream = connect(polygon_ws_url.clone(), &polygon_api_key).await;

        let subscription_params = [
            "XT.BTC-USD",  // Trades for BTC-USD
            "XA.BTC-USD",  // Minute aggregates for BTC-USD

            "XT.ETH-USD",  // Trades for ETH-USD
            "XA.ETH-USD",  // Minute aggregates for ETH-USD

            "XT.LINK-USD", // Trades for LINK-USD
            "XA.LINK-USD", // Minute aggregates for LINK-USD
        ];

        subscribe_to_polygon_events_with_params(&mut polygon_ws_stream, &subscription_params).await;

        // Process incoming messages and send them to the dataflow through the channel
        consume_polygon_stream(&mut polygon_ws_stream, polygon_event_sender).await;
    });

    // This task executes the Timely Dataflow
    let polygon_dataflow_task = tokio::spawn(async move {
        // Initialize and execute the dataflow
        thread::spawn(move || {
            execute_from_args(env::args(), move |worker| {
                let input_handle: InputHandle<i64, PolygonEventTypes> = InputHandle::new();

                // Continuously read from the channel and process messages
                loop {
                    // Consume the crossbeam channel with messages from the websocket
                    match polygon_event_receiver.recv() {
                        Ok(event) => {

                            // Definition of hardcoded filters

                            // sample XT Trade Event filters
                            let mut criteria_by_pair = HashMap::new();

                            let btc_criteria = FilterCriteria {
                                field: "size".to_string(),
                                operation: ">".to_string(),
                                value: FilterValue::Number(0.3),
                            };

                            let eth_criteria = FilterCriteria {
                                field: "size".to_string(),
                                operation: ">".to_string(),
                                value: FilterValue::Number(2.0),
                            };

                            let link_criteria = FilterCriteria {
                                field: "size".to_string(),
                                operation: ">".to_string(),
                                value: FilterValue::Number(2.0),
                            };

                            // Insert criteria into the HashMap
                            criteria_by_pair.insert("BTC-USD".to_string(), vec![btc_criteria]);
                            criteria_by_pair.insert("ETH-USD".to_string(), vec![eth_criteria]);
                            criteria_by_pair.insert("LINK-USD".to_string(), vec![link_criteria]);

                            let param_filter = Arc::new(ParameterizedFilter::new(criteria_by_pair));

                            //define Event filters
                            let mut filters = EventFilters::new();

                            filters.add_filter(param_filter);

                            run_filtered_polygon_dataflow(event, worker, filters, db_session_manager.clone());

                            worker.step(); // Drive the dataflow forward
                        }
                        Err(_) => {
                            break; // Exit loop if the sender is dropped/disconnected
                        }
                    }
                }
            })
                .unwrap();
        });
    });

    // Wait for all tasks to complete - they won't!
    let _ = tokio::try_join!(polygon_ws_message_processing_task, polygon_dataflow_task, ws_server_task);
    Ok(())
}
