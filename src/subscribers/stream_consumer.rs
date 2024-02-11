use async_trait::async_trait;
use crossbeam_channel::Sender;
use futures_util::StreamExt;
use serde_json::Value;
use std::error::Error as StdError; // Renamed for clarity
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};

use crate::subscribers::websocket_event_types::WebsocketEventTypes;

// Standalone async function for consuming the stream.
pub async fn consume_stream<T: WebsocketEventTypes + Send + 'static>(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    tx: Sender<T>,
) {
    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    let event_jsons: Result<Value, _> = serde_json::from_str(&text);
                    match event_jsons {
                        Ok(events) => {
                            if events.is_array() {
                                for event in events.as_array().unwrap() {
                                    println!("ATTEMPTING TO DESERIALIZE {:?}", event);
                                    match T::deserialize_event(&event) {
                                        Ok(event) => {
                                            tx.send(event).expect("Failed to send event to the dataflow");
                                        },
                                        Err(e) => {
                                            eprintln!("Error deserializing message: {:?}", e)
                                        },
                                    }
                                }
                            } else {
                                // Handle single event deserialization
                                println!("ATTEMPTING TO DESERIALIZE {:?}", events);
                                match T::deserialize_event(&events) {
                                    Ok(event) => {
                                        tx.send(event).expect("Failed to send event to the dataflow");
                                    },
                                    Err(e) => eprintln!("Error deserializing message: {:?}", e),
                                }
                            }
                        }
                        Err(e) => eprintln!("Error parsing JSON: {:?}", e),
                    }
                }
            }
            Err(e) => eprintln!("Error receiving message: {:?}", e),
        }
    }
}
