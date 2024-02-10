use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use url::Url;
use serde_json::json;


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

    pub async fn subscribe(&mut self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, params: &str) {
        let subscribe_message = Message::Text(
            serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_subscribe",
            "params": [params, {
                // Optionally add filters here
            }]
        }))
                .expect("Failed to serialize subscription message")
        );

        ws_stream.send(subscribe_message).await.expect("Failed to subscribe to alchemy_minedTransactions");
        println!("Subscribed to Alchemy mined transactions");
    }

    // Add a processing function here if needed
}
