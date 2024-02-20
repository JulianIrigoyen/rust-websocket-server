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
extern crate serde;
extern crate serde_derive;
extern crate timely;

use std::{env, thread};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crossbeam_channel::{bounded, Sender};
use dotenv::dotenv;
use ethers::{
    prelude::abigen,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use timely::dataflow::InputHandle;
use timely::dataflow::operators::{Filter, Input, Inspect};
use timely::execute_from_args;
use timely::worker::Worker;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream,
};
use url::Url;

use crate::db::db_session_manager::DbSessionManager;
use crate::models::alchemy::alchemy_event_types::AlchemyEventTypes;
use crate::models::alchemy::alchemy_mined_transaction_data::AlchemyMinedTransactionData;
use crate::models::binance::binance_event_types::BinanceEventTypes;
use crate::models::polygon::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon::polygon_crypto_trade_data::PolygonCryptoTradeData;
use crate::models::polygon::polygon_event_types::PolygonEventTypes;
use crate::server::ws_server;
use crate::subscriber::websocket_subscriber::{WebSocketSubscriber, AlchemySubscriptionBuilder, PolygonSubscriptionBuilder, BinanceSubscriptionBuilder, AuthMethod};
use crate::trackers::alchemy_whale_tracker::AlchemyWhaleTracker;
use crate::trackers::polygon_price_action_tracker::PolygonPriceActionTracker;
use crate::util::event_filters::{
    EventFilters, FilterCriteria, FilterValue, ParameterizedFilter,
};
use crate::subscriber::consume_stream::{consume_stream};
use crate::trackers::binance::rsi_tracker::RsiTracker;

mod db;
mod util;
mod models;
mod server;
mod http;
mod trackers;
mod schema;
mod subscriber;

/**

    Rust Websocket Server : (How it started)
       Establishes a WebSocket connection with Polygon.io's API, authenticates, and subscribes to various trading data streams.
        This includes trades, quotes, level 2 books, and aggregate data on a per-minute or per-second basis.
                                                                                                                    Thu Feb 8 04:56:24 2024 -0300

    Rust Websocket Consumer : (How its going)
        Establishes websocket connections with N platforms to aggregate crypto market data for scalp trading.
                                                                                                                    Sun Feb 11 18:49:36 2024 -0300

 */

fn run_filtered_alchemy_dataflow(
    event: AlchemyEventTypes,
    worker: &mut Worker<timely::communication::Allocator>,
    filters: EventFilters,
    db_session_manager: Arc<DbSessionManager>,
) {
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<AlchemyEventTypes>();

        // println!("Processing Alchemy Event: {:?}", event);
        input_handle.send(event); // Send the event into the dataflow

        let alchemy_whale_tracker =
            AlchemyWhaleTracker::new(db_session_manager.clone(), 100.0);

        stream.inspect(move |event| {
            alchemy_whale_tracker.apply(event);
        });

        // for filter in &filters.filters {
        //     let filter_clone = filter.clone();
        //     let db_session_manager_cloned = db_session_manager.clone();
        //
        //     stream
        //         .filter(move |x| filter_clone.apply(x))
        //         .inspect(move |x| {
        //             println!("Filtered Alchemy Event: {:?}", x);
        //
        //             match db_session_manager_cloned.persist_event(x) {
        //                 Ok(_) => println!("Alchemy Event successfully persisted."),
        //                 Err(e) => eprintln!("Error persisting alchemy event: {:?}", e),
        //             }
        //         });
        // }

        input_handle.advance_to(1);
    });
}


fn run_filtered_polygon_dataflow(
    event: PolygonEventTypes,
    worker: &mut Worker<timely::communication::Allocator>,
    filters: EventFilters,
    db_session_manager: Arc<DbSessionManager>,
) {
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<PolygonEventTypes>();

        // stream.inspect(|x| println!("Data: {:?}", x));

        // Create an input handle and stream for PolygonEventTypes
        input_handle.send(event); // Send the event into the dataflow

        // Create a new instance of the price movement tracker with very low threshold to see it in action
        let price_movement_tracker = PolygonPriceActionTracker::new(0.0001);
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
                .inspect(move |x| { // Use `move` to capture filtered values

                    // println!("Filtered Polygon Event: {:?}", x);

                    match db_session_manager_cloned.persist_event(x) {
                        Ok(_) => (),
                        // println!("Polygon Event successfully persisted."),
                        Err(e) => eprintln!("Error persisting polygon event: {:?}", e),
                    }
                });
        }

        input_handle.advance_to(1); // Make sure to advance the input handle
    });
}

fn run_filtered_binance_dataflow(
    event: BinanceEventTypes,
    worker: &mut Worker<timely::communication::Allocator>,
    filters: EventFilters,
    db_session_manager: Arc<DbSessionManager>,
    rsi_tracker: Arc<Mutex<RsiTracker>>,
) {
    worker.dataflow(|scope| {
        let (mut input_handle, stream) = scope.new_input::<BinanceEventTypes>();

        stream.inspect(move |event| {
            let rsi_tracker_clone = Arc::clone(&rsi_tracker);
            if let BinanceEventTypes::Kline(kline) = event {
                if kline.symbol == "BTCUSDT" {
                    match kline.kline.interval.as_str() {
                        "1s" | "5m" | "15m" => {
                            // Lock the RsiTracker for each event to safely update its state
                            let mut tracker = rsi_tracker_clone.lock().unwrap();
                            tracker.apply_kline(kline);

                            if let Some(rsi) = tracker.get_rsi(&kline.symbol, &kline.kline.interval) {
                                println!("Updated RSI for {} at {} interval: {}", kline.symbol, kline.kline.interval, rsi);
                            }
                        },
                        _ => {} // Ignore other intervals
                    }
                }
            }
        });

        input_handle.send(event);
        input_handle.advance_to(1);
    });
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_session_manager = Arc::new(DbSessionManager::new(&database_url));

    // Websocket Server Initialization
    let ws_host = env::var("WS_SERVER_HOST").expect("WS_HOST must be set");
    let ws_port = env::var("WS_SERVER_PORT").expect("WS_PORT must be set");
    let ws_server_task = tokio::spawn(async move {
        let ws_server = ws_server::WebSocketServer::new(ws_host, ws_port);
        ws_server.run().await
    });

    /// Ethereum Alchemy websocket processing task
    // let alchemy_http_url = env::var("ALCHEMY_HTTP_URL").expect("ALCHEMY_HTTP_URL must be set");
    // let alchemy_ws_url = env::var("ALCHEMY_WS_URL").expect("ALCHEMY_WS_URL must be set");
    // let alchemy_api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set");
    //
    // // Setup channels for the specific event types we want
    // let (alchemy_event_sender, alchemy_event_receiver) = bounded::<AlchemyEventTypes>(5000);
    //
    // let alchemy_subscriber = WebSocketSubscriber::<AlchemySubscriptionBuilder>::new(
    //     alchemy_ws_url,
    //     Some(alchemy_api_key),
    //     AuthMethod::QueryParam,
    //     AlchemySubscriptionBuilder,
    // );
    //
    // let alchemy_params: Vec<(&str, Vec<String>)> = vec![
    //     ("ethereum", vec!["alchemy_minedTransactions".to_string()])
    // ];
    // let mut alchemy_ws_stream = alchemy_subscriber.connect().await?;
    // alchemy_subscriber.subscribe(&mut alchemy_ws_stream, &alchemy_params).await?;
    //
    // let alchemy_ws_message_processing_task = tokio::spawn(async move {
    //     consume_stream::<AlchemyEventTypes>(&mut alchemy_ws_stream, alchemy_event_sender).await;
    // });
    //
    // /// Ethereum Alchemy Dataflow task
    // let alchemy_db_session_manager = db_session_manager.clone();
    // let alchemy_dataflow_task = tokio::spawn(async move {
    //     thread::spawn(move || {
    //         execute_from_args(env::args(), move |worker| {
    //             let input_handle: InputHandle<i64, AlchemyEventTypes> = InputHandle::new();
    //
    //             // Continuously read from the channel and process messages
    //             loop {
    //                 // Consume the crossbeam channel with messages from the websocket
    //                 match alchemy_event_receiver.recv() {
    //                     Ok(event) => {
    //                         run_filtered_alchemy_dataflow(event, worker, EventFilters::new(), alchemy_db_session_manager.clone());
    //                         worker.step(); // Drive the dataflow forward
    //                     }
    //                     Err(_) => {
    //                         break; // Exit loop if the sender is dropped/disconnected
    //                     }
    //                 }
    //             }
    //         }).unwrap();
    //     });
    // });
    //
    //
    //
    // /// PolygonIO Websocket processing task
    // let polygon_ws_url_string = env::var("POLYGON_WS_URL").expect("Expected POLYGON_API_KEY to be set");
    // let polygon_api_key = env::var("POLYGON_API_KEY").expect("Expected POLYGON_API_KEY to be set");
    // let polygon_subscriber = WebSocketSubscriber::<PolygonSubscriptionBuilder>::new(
    //     polygon_ws_url_string,
    //     Some(polygon_api_key),
    //     AuthMethod::Message, // Polygon uses the API key in an authentication message
    //     PolygonSubscriptionBuilder,
    // );
    //
    // let polygon_params: Vec<(&str, Vec<String>)> = vec![
    //     ("polygon", vec![
    //         "XT.BTC-USD", "XA.BTC-USD",
    //         "XT.ETH-USD", "XA.ETH-USD",
    //         "XT.LINK-USD", "XA.LINK-USD",
    //         "XT.SOL-USD", "XA.SOL-USD",
    //     ].iter().map(|s| s.to_string()).collect()),
    // ];
    // let mut polygon_ws_stream = polygon_subscriber.connect().await?;
    //
    // // Setup channels for the specific event types we want
    // let (polygon_event_sender, polygon_event_receiver) = bounded::<PolygonEventTypes>(5000);
    // polygon_subscriber.subscribe(&mut polygon_ws_stream, &polygon_params).await?;
    //
    // let polygon_ws_message_processing_task = tokio::spawn(async move {
    //     consume_stream::<PolygonEventTypes>(&mut polygon_ws_stream, polygon_event_sender).await;
    // });
    //
    // /// PolygonIO Dataflow Task
    // let polygon_dataflow_task = tokio::spawn(async move {
    //     // Initialize and execute the dataflow
    //     thread::spawn(move || {
    //         execute_from_args(env::args(), move |worker| {
    //             let input_handle: InputHandle<i64, PolygonEventTypes> = InputHandle::new();
    //
    //             // Continuously read from the channel and process messages
    //             loop {
    //                 // Consume the crossbeam channel with messages from the websocket
    //                 match polygon_event_receiver.recv() {
    //                     Ok(event) => {
    //
    //                         // Definition of hardcoded filters
    //
    //                         // sample XT Trade Event filters
    //                         let mut criteria_by_pair = HashMap::new();
    //
    //                         let btc_criteria = FilterCriteria {
    //                             field: "size".to_string(),
    //                             operation: ">".to_string(),
    //                             value: FilterValue::Number(1.0),
    //                         };
    //
    //                         let eth_criteria = FilterCriteria {
    //                             field: "size".to_string(),
    //                             operation: ">".to_string(),
    //                             value: FilterValue::Number(10.0),
    //                         };
    //
    //                         let link_criteria = FilterCriteria {
    //                             field: "size".to_string(),
    //                             operation: ">".to_string(),
    //                             value: FilterValue::Number(100.0),
    //                         };
    //
    //                         let sol_criteria = FilterCriteria {
    //                             field: "size".to_string(),
    //                             operation: ">".to_string(),
    //                             value: FilterValue::Number(30.0),
    //                         };
    //
    //                         // Insert criteria into the HashMap
    //                         criteria_by_pair.insert("BTC-USD".to_string(), vec![btc_criteria]);
    //                         criteria_by_pair.insert("ETH-USD".to_string(), vec![eth_criteria]);
    //                         criteria_by_pair.insert("LINK-USD".to_string(), vec![link_criteria]);
    //                         criteria_by_pair.insert("SOL-USD".to_string(), vec![sol_criteria]);
    //
    //                         let param_filter = Arc::new(ParameterizedFilter::new(criteria_by_pair));
    //
    //                         //define Event filters
    //                         let mut filters = EventFilters::new();
    //
    //                         filters.add_filter(param_filter);
    //
    //                         run_filtered_polygon_dataflow(event, worker, filters, db_session_manager.clone());
    //
    //                         worker.step(); // Drive the dataflow forward
    //                     }
    //                     Err(_) => {
    //                         break; // Exit loop if the sender is dropped/disconnected
    //                     }
    //                 }
    //             }
    //         })
    //             .unwrap();
    //     });
    // });

    // Binance Websocket processing task (Old)
    // let binance_ws_url_string = env::var("BINANCE_WS_URL").expect("Expected BINANCE_WS_URL to be set");
    // let binance_api_key = env::var("BINANCE_API_KEY").expect("Expected BINANCE_API_KEY to be set");
    // let binance_subscriber = WebSocketSubscriber::<BinanceSubscriptionBuilder>::new(
    //     binance_ws_url_string,
    //     Some(binance_api_key),
    //     AuthMethod::QueryParam,
    //     BinanceSubscriptionBuilder,
    // );

    let binance_params: Vec<(&str, Vec<String>)> = vec![
        // Streams !
        // !miniTicker@arr -> all market mini tickers
        // <symbol>@ticker -> single symbol ticker
        // <symbol>@avgPrice -> average price
        // !ticker@arr -> all market tickers
        // <symbol>@ticker_<window_size> -> single symbol rolling w -- Window Sizes: 1h,4h,1d
        // !ticker_<window-size>@arr -> all market rolling w
        // <symbol>@bookTicker -> symbol order book : Pushes any update to the best bid or ask's price or quantity in real-time for a specified symbol.
        // <symbol>@depth<levels> (1000ms) / <symbol>@depth<levels>@100ms -> partial book depth : Top bids and asks, Valid are 5, 10, or 20.



        // Partial Book Depth Streams for ETH, SOL, and BTC
        ("ETHUSDT", vec!["depth5".into(), "depth10".into(), "depth20".into()]),
        ("SOLUSDT", vec!["depth5".into(), "depth10".into(), "depth20".into()]),
        ("BTCUSDT", vec!["depth5".into(), "depth10".into(), "depth20".into()]),

        // Trade Streams for ETH, SOL, and BTC
        ("ETHUSDT", vec!["trade".into()]),
        ("SOLUSDT", vec!["trade".into()]),
        ("BTCUSDT", vec!["trade".into()]),

        // Kline/Candlestick Streams for ETH, SOL, and BTC at 1m and 5m intervals
        // ("ETHUSDT", vec!["kline_1m", "kline_5m"].into_iter().map(String::from).collect()),
        // ("SOLUSDT", vec!["kline_1m", "kline_5m"].into_iter().map(String::from).collect()),
        // ("BTCUSDT", vec!["kline_1m", "kline_5m"].into_iter().map(String::from).collect()),
    ];
    //
    // let mut binance_ws_stream = binance_subscriber.connect().await?;
    // binance_subscriber.subscribe(&mut binance_ws_stream, &binance_params).await?;


    // Connect to the Binance WebSocket server
    let binance_ws_url = "wss://stream.binance.com:9443/stream";
    let (mut binance_ws_stream, _) = connect_async(binance_ws_url).await?;
    println!("Connected to Binance WebSocket");

    let subscriber = WebSocketSubscriber::<BinanceSubscriptionBuilder>::new(
        binance_ws_url.to_string(),
        None,
        AuthMethod::None,
        BinanceSubscriptionBuilder,
    );


    // ("ETHUSDT", vec!["ethusdt@depth5".into(), "depth10".into(), "ethusdt@depth20".into()]),
    // ("SOLUSDT", vec!["solusdt@depth5".into(), "depth10".into(), "sol@usdtdepth20".into()]),
    // ("BTCUSDT", vec!["btcusdt@depth5".into(), "btcusdt@depth10".into(), "btcusdt@depth20".into()]),
    //
    // // Trade Streams for ETH, SOL, and BTC
    // ("ETHUSDT", vec!["ethusdt@trade".into()])
    // ("SOLUSDT", vec!["solusdt@trade".into()])
    // ("BTCUSDT", vec!["btcusdt@trade".into()])

    // Subscribe to binance streams
    subscriber.binance_subscribe_streams(&mut binance_ws_stream,
                                         vec![
                                             "btcusdt@kline_1min".into(),
                                             "solusdt@kline_1min".into(),
                                             "ethusdt@kline_1min".into(),
                                             "btcusdt@depth5".into(),
                                             "ethusdt@depth5".into(),
                                             "solusdt@depth5".into(),
                                             // "btcusdt@depth20".into(),
                                             // "ethusdt@depth20".into(),
                                             // "solusdt@depth20".into(),
                                         ]
    ).await?;

    // subscriber.binance_subscribe_streams(&mut binance_ws_stream,
    //                                      vec!["btcusdt@depth5".into(), "btcusdt@depth10".into(), "btcusdt@depth20".into()]
    //                                      //vec!["ethusdt".into(), "solusdt@trade".into(), "btcusdt@trade".into()]
    // ).await?;

    // sample unsub
    //subscriber.binance_unsubscribe_streams(&mut ws_stream, vec!["btcusdt@depth".into()]).await?;

    // List current subscriptions
    //subscriber.binance_list_subscriptions(&mut ws_stream).await?;

    let (binance_event_sender, binance_event_receiver) = bounded::<BinanceEventTypes>(5000);

    let binance_ws_message_processing_task = tokio::spawn(async move {
        consume_stream::<BinanceEventTypes>(&mut binance_ws_stream, binance_event_sender).await;
    });

    /// Binance Dataflow task
    let rsi_tracker = Arc::new(Mutex::new(RsiTracker::new()));

    let rsi_tracker_intervals = vec!["1s", "5m", "15m"];

    let intervals_with_periods = vec![
        ("1s", 14),
        ("5m", 14),
        ("15m", 14),
    ];

    // Configure intervals for BTCUSDT
    {
        let mut tracker = rsi_tracker.lock().unwrap();
        // tracker.set_intervals_for_symbol("BTCUSDT", rsi_tracker_intervals);
        // tracker.set_intervals_for_symbol("ETHUSDT", vec!["1s", "5m", "15m"]);
        // tracker.set_intervals_for_symbol("SOLUSDT", vec!["1s", "5m", "15m"]);
        // tracker.set_intervals_for_symbol("DOGEUSDT", vec!["1s", "5m", "15m"]);
    }

    let binance_db_session_manager = db_session_manager.clone();
    let binance_dataflow_task = tokio::spawn(async move {
        thread::spawn(move || {
            execute_from_args(env::args(), move |worker| {
                let input_handle: InputHandle<i64, BinanceEventTypes> = InputHandle::new();

                // Continuously read from the channel and process messages
                loop {
                    // Consume the crossbeam channel with messages from the websocket
                    match binance_event_receiver.recv() {
                        Ok(event) => {
                            run_filtered_binance_dataflow(event, worker, EventFilters::new(), binance_db_session_manager.clone(), Arc::clone(&rsi_tracker));
                            worker.step(); // Drive the dataflow forward
                        }
                        Err(_) => {
                            break; // Exit loop if the sender is dropped/disconnected
                        }
                    }
                }
            }).unwrap();
        });
    });


    // Wait for all tasks to complete
    let _ = tokio::try_join!(
        ws_server_task,
        // alchemy_ws_message_processing_task,
        // alchemy_dataflow_task,
        // polygon_ws_message_processing_task,
        // polygon_dataflow_task,
        binance_ws_message_processing_task,
        binance_dataflow_task
    );

    Ok(())
}