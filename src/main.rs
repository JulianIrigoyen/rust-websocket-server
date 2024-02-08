#![allow(unused_variables)]
extern crate timely;

use std::sync::{Arc, Mutex};
use crossbeam_channel::{bounded, Sender, Receiver};

use std::thread;

use futures_util::{sink::SinkExt, stream::StreamExt};
use timely::communication::allocator::Generic;
use timely::dataflow::InputHandle;
use timely::dataflow::operators::{Exchange, Input, Inspect, Probe, ToStream};
use timely::execute_from_args;
use timely::worker::Worker;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use url::Url;

use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_crypto_level2_book_data::PolygonCryptoLevel2BookData;
use crate::models::polygon_crypto_quote_data::PolygonCryptoQuoteData;
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;
use crate::models::polygon_event_types::PolygonEventTypes;

mod models;

// Function to connect, authenticate, and subscribe to WebSocket
async fn connect(ws_url: Url, api_key: &str) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    let (mut ws_stream, _) = connect_async(ws_url.clone()).await.expect("Failed to connect to WebSocket");
    println!("Connected to {}", ws_url);
    // Authenticate
    let auth_message = Message::Text(format!(r#"{{"action":"auth","params":"{}"}}"#, api_key));
    ws_stream.send(auth_message).await.expect("Failed to authenticate");
    println!("Authenticated with {}", ws_url);
    ws_stream
}

async fn subscribe_with_parms(ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, params: &[&str]) {
    for &param in params {
        let sub = Message::Text(format!(r#"{{"action":"subscribe","params":"{}"}}"#, param));
        ws_stream.send(sub).await.expect("Failed to send subscribe message");
        println!("Subscribed to {}", param);
    }
}

async fn subscribe_to_all_polygon_events(ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    // Subscription message for all aggregates per minute
    let all_aggregates_per_min_sub = Message::Text(r#"{"action":"subscribe","params":"XA.*"}"#.to_string());
    ws_stream.send(all_aggregates_per_min_sub).await.expect("Failed to send subscribe message for aggregates per minute");
    println!("Subscribed to all aggregates per minute");

    // Subscription message for all aggregates per second
    let all_aggregates_per_second_sub = Message::Text(r#"{"action":"subscribe","params":"XAS.*"}"#.to_string());
    ws_stream.send(all_aggregates_per_second_sub).await.expect("Failed to send subscribe message for aggregates per second");
    println!("Subscribed to all aggregates per second");

    // Subscription message for all trades
    let all_trades_sub = Message::Text(r#"{"action":"subscribe","params":"XT.*"}"#.to_string());
    ws_stream.send(all_trades_sub).await.expect("Failed to send subscribe message for all trades");
    println!("Subscribed to all trades");

    // Subscription message for all quotes
    let all_quotes_sub = Message::Text(r#"{"action":"subscribe","params":"XQ.*"}"#.to_string());
    ws_stream.send(all_quotes_sub).await.expect("Failed to send subscribe message for all quotes");
    println!("Subscribed to all quotes");

    //Subscription message for all level 2 books
    let all_level_2_book_sub = Message::Text(r#"{"action":"subscribe","params":"XL2.*"}"#.to_string());
    ws_stream.send(all_level_2_book_sub).await.expect("Failed to send subscribe message for all level 2 books");
    println!("Subscribed to all level 2 books");

    // Subscription message for all fair market values - - TODO REQUIRES BUSINESS
    // let all_fair_market_value_sub = Message::Text(r#"{"action":"subscribe","params":"FMV.*"}"#.to_string());
    // ws_stream.send(all_fair_market_value_sub).await.expect("Failed to send subscribe message for all fair market values");
    // println!("Subscribed to all fair market values");
}

async fn process_websocket_messages(ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, tx: Sender<PolygonEventTypes>) {
    println!("Starting to process WebSocket messages");

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    println!("Received WebSocket message: {}", text);

                    if let Some(event) = deserialize_message(&text) {
                        println!("SENDING");
                        tx.send(event).expect("Failed to send event to dataflow");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
}

fn deserialize_message(text: &str) -> Option<PolygonEventTypes> {
    println!("Deserializing Message: {}", text);

    // Attempt to deserialize trade data list
    if let Ok(trade_data_list) = serde_json::from_str::<Vec<PolygonCryptoTradeData>>(text) {
        if let Some(first_trade_data) = trade_data_list.get(0) {
            // println!("Deserialized a trade event");
            return Some(PolygonEventTypes::XtTrade(first_trade_data.clone()));
        }
    }

    // Attempt to deserialize aggregate data
    if let Ok(aggregate_data_list) = serde_json::from_str::<Vec<PolygonCryptoAggregateData>>(text) {
        if let Some(data) = aggregate_data_list.get(0) {
            // println!("Deserialized an aggregate event");
            return Some(PolygonEventTypes::XaAggregateMinute(data.clone()));
        }
    }

    // Attempt to deserialize quote data
    if let Ok(quote_data_list) = serde_json::from_str::<Vec<PolygonCryptoQuoteData>>(text) {
        if let Some(data) = quote_data_list.get(0) {
            // println!("Deserialized a quote event");
            return Some(PolygonEventTypes::XqQuote(data.clone()));
        }
    }

    // Attempt to deserialize level 2 book data
    if let Ok(level2_book_data) = serde_json::from_str::<Vec<PolygonCryptoLevel2BookData>>(text) {
        if let Some(data) = level2_book_data.get(0) {
            // println!("Deserialized a level 2 book event");
            return Some(PolygonEventTypes::Xl2Level2book(data.clone()));
        }
    }

    // If no known type matches, log and return None
    println!("Message does not match known event types");
    None
}


#[tokio::main]
async fn main() {
    let (sender, receiver) = bounded::<PolygonEventTypes>(5000); // Adjust buffer size as needed

    let polygon_ws_url = Url::parse("wss://socket.polygon.io/crypto").expect("Invalid WebSocket URL");
    let mut polygon_ws_stream = connect(polygon_ws_url, "EdpUuqshn08VIPoOozwCg6RSE9dVXmxp").await;


    subscribe_to_all_polygon_events(&mut polygon_ws_stream).await;

    thread::spawn(move || {
        execute_from_args(std::env::args(), move |worker| {
            let mut input_handle = InputHandle::new();

            // Build the dataflow
            worker.dataflow(|scope| {
                scope.input_from(&mut input_handle)
                    .inspect(|x| println!("Received in dataflow: {:?}", x));
            });

            // Continuously read from the channel and process messages
            loop {
                println!("RECEIVING");
                match receiver.recv() {
                    Ok(event) => {

                        // Send the event to the dataflow
                        input_handle.send(event);
                        // Signal that we are done with this epoch
                        input_handle.advance_to(input_handle.epoch() + 1);
                        worker.step(); // Make sure to drive the dataflow forward
                    }
                    Err(_) => {
                        // The channel has been disconnected, likely because the sending side is dropped
                        // This is a good place to clean up or exit the loop if necessary
                        break;
                    }
                }
            }
        }).unwrap();
    });


    process_websocket_messages(&mut polygon_ws_stream, sender).await;


    let default_symbols = vec![
        "BTC", "ETH", "BNB", "XRP", "ADA",
        "SOL", "DOT", "LTC", "DOGE", "LINK",
        "UNI", "BCH", "AVAX", "ALGO", "XLM",
        "VET", "ICP", "FIL", "TRX",
    ];
}
