use futures_util::SinkExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use url::Url;

pub struct AlchemySubscriber {
    ws_url: String,
}

impl AlchemySubscriber {
    pub fn new(url: String, api_key: String) -> Self {
        let ws_url = format!("{}{}", url, api_key);
        AlchemySubscriber { ws_url }
    }

    pub async fn connect(&self) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let url = Url::parse(&self.ws_url).expect("Failed to parse WebSocket URL");
        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect to Alchemy WebSocket");
        println!("Connected to Alchemy WebSocket");
        ws_stream
    }

    pub async fn subscribe(&mut self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, method: &str, params: Vec<serde_json::Value>) {
        let subscribe_message = Message::Text(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_subscribe",
                "params": params
            }))
                .expect("Failed to serialize subscription message")
        );

        ws_stream.send(subscribe_message).await.expect("Failed to subscribe");
        let param_strs: Vec<String> = params.iter().map(|p| p.to_string()).collect();
        println!("Subscribed to Alchemy Websocket events: {} ", param_strs.join(", "));
    }
}
