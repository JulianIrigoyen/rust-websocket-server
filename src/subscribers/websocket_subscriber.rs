use std::error::Error;

use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use url::Url;

pub struct WebSocketSubscriber {
    ws_url: String,
    subscribe_messages: Vec<Message>,
}

impl WebSocketSubscriber {
    pub fn new(ws_url: String, subscribe_messages: Vec<Message>) -> Self {
        Self { ws_url, subscribe_messages }
    }

    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
        let url = Url::parse(&self.ws_url)?;
        let (ws_stream, _) = connect_async(url).await?;
        println!("Connected to WebSocket :: {}", self.ws_url);
        Ok(ws_stream)
    }

    pub async fn subscribe(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn Error>> {
        for message in &self.subscribe_messages {
            ws_stream.send(message.clone()).await?;
        }
        println!("Subscribed with provided messages :: {:?}", self.subscribe_messages);
        Ok(())
    }
}