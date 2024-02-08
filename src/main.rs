#![allow(unused_variables)]
extern crate timely;

mod models;

use timely::dataflow::operators::{Input, Exchange, Inspect, Probe};
use timely::dataflow::InputHandle;
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use tokio::net::TcpStream;
use url::Url;
use crate::models::polygon_crypto_aggregate_data::PolygonCryptoAggregateData;
use crate::models::polygon_crypto_trade_data::PolygonCryptoTradeData;

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
    // let all_aggregates_per_min_sub = Message::Text(r#"{"action":"subscribe","params":"XA.*"}"#.to_string());
    // ws_stream.send(all_aggregates_per_min_sub).await.expect("Failed to send subscribe message for aggregates per minute");
    // println!("Subscribed to all aggregates per minute");
    //
    // // Subscription message for all aggregates per second
    // let all_aggregates_per_second_sub = Message::Text(r#"{"action":"subscribe","params":"XAS.*"}"#.to_string());
    // ws_stream.send(all_aggregates_per_second_sub).await.expect("Failed to send subscribe message for aggregates per second");
    // println!("Subscribed to all aggregates per second");

    // Subscription message for all trades
    let all_trades_sub = Message::Text(r#"{"action":"subscribe","params":"XT.*"}"#.to_string());
    ws_stream.send(all_trades_sub).await.expect("Failed to send subscribe message for all trades");
    println!("Subscribed to all trades");
    //
    // // Subscription message for all quotes
    // let all_quotes_sub = Message::Text(r#"{"action":"subscribe","params":"XQ.*"}"#.to_string());
    // ws_stream.send(all_quotes_sub).await.expect("Failed to send subscribe message for all quotes");
    // println!("Subscribed to all quotes");
    //
    // //Subscription message for all level 2 books
    // let all_level_2_book_sub = Message::Text(r#"{"action":"subscribe","params":"XL2.*"}"#.to_string());
    // ws_stream.send(all_level_2_book_sub).await.expect("Failed to send subscribe message for all level 2 books");
    // println!("Subscribed to all level 2 books");

    // Subscription message for all fair market values - - REQUIRES BUSINESS
    // let all_fair_market_value_sub = Message::Text(r#"{"action":"subscribe","params":"FMV.*"}"#.to_string());
    // ws_stream.send(all_fair_market_value_sub).await.expect("Failed to send subscribe message for all fair market values");
    // println!("Subscribed to all fair market values");
}

async fn process_websocket_messages(ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    // Deserialize the received JSON text into PolygonCryptoTradeData
                    if let Ok(trade_data) = serde_json::from_str::<PolygonCryptoTradeData>(&text) {
                        println!("Parsed trade data: {} : {}", trade_data.pair, trade_data.price);
                    }

                    if let Ok(aggregate_data) = serde_json::from_str::<PolygonCryptoAggregateData>(&text) {
                        println!("Parsed aggregate data: {} : Hi -> {}", aggregate_data.pair, aggregate_data.high);
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


#[tokio::main]
async fn main() {
    let default_symbols = vec![
        "BTC", "ETH", "BNB", "XRP", "ADA",
        "SOL", "DOT", "LTC", "DOGE", "LINK",
        "UNI", "BCH", "AVAX", "ALGO", "XLM",
        "VET", "ICP", "FIL", "TRX",
    ];

    let polygon_ws_url = Url::parse("wss://socket.polygon.io/crypto").expect("Invalid WebSocket URL");

    let mut polygon_ws_stream = connect(polygon_ws_url, "EdpUuqshn08VIPoOozwCg6RSE9dVXmxp").await;

    // subscribe_with_parms(&mut polygon_ws_stream, &default_symbols).await;

    subscribe_to_all_polygon_events(&mut polygon_ws_stream).await;

    while let Some(message) = polygon_ws_stream.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    // Print all received messages
                    // println!("Received message: {:?}", text);
                    // Deserialize the received JSON text into PolygonCryptoTradeData
                    if let Ok(trade_data) = serde_json::from_str::<Vec<PolygonCryptoTradeData>>(&text) {
                        for t in trade_data {
                            if t.pair == "BTC-USD" {
                                println!("fOUND bTc GERT: {} : {} x {}", t.pair, t.price, t.size);
                            }

                        }
                    }

                    if let Ok(aggregate_data) = serde_json::from_str::<PolygonCryptoAggregateData>(&text) {
                        println!("Parsed aggregate data: {} : Hi -> {}", aggregate_data.pair, aggregate_data.high);
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
