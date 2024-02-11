use std::error::Error;

use futures_util::SinkExt;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
use url::Url;

pub trait SubscriptionBuilder {
    fn build_subscription_messages(params: &[(&str, Vec<String>)]) -> Vec<Message>;
}

pub enum AuthMethod {
    None,
    QueryParam,
    Message,
}

pub struct WebSocketSubscriber<B: SubscriptionBuilder> {
    ws_url: String,
    api_key: Option<String>, // API key is now optional
    auth_method: AuthMethod,
    builder: B,
}

impl<B: SubscriptionBuilder> WebSocketSubscriber<B> {
    pub fn new(ws_url: String, api_key: Option<String>, auth_method: AuthMethod, builder: B) -> Self {
        Self { ws_url, api_key, auth_method, builder }
    }

    pub async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {

        // println!("CONNECTING {:?}", self.ws_url);
        let final_url = match self.auth_method {
            AuthMethod::QueryParam => {
                if let Some(ref key) = self.api_key {
                    format!("{}{}", self.ws_url, key)
                } else {
                    self.ws_url.clone()
                }
            },
            _ => self.ws_url.clone(),
        };

        let url = Url::parse(&final_url)?;
        let (mut ws_stream, _) = connect_async(url).await?;
        println!("Connected to WebSocket :: {}", final_url);

        if let AuthMethod::Message = self.auth_method {
            self.authenticate(&mut ws_stream).await?;
        }

        Ok(ws_stream)
    }

    async fn authenticate(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn Error>> {
        if let Some(ref api_key) = self.api_key {
            let auth_message = Message::Text(json!({
                "action": "auth",
                "params": api_key
            }).to_string());

            ws_stream.send(auth_message).await?;
            println!("Authentication message sent");
        }

        Ok(())
    }


    pub async fn subscribe(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, params: &[(&str, Vec<String>)]) -> Result<(), Box<dyn Error>> {
        let messages = B::build_subscription_messages(params);
        for message in messages {
            println!("Subscribing to {} with provided messages :: {:?}", self.ws_url, message);
            ws_stream.send(message).await?;
        }

        Ok(())
    }
}

pub struct PolygonSubscriptionBuilder;
pub struct BinanceSubscriptionBuilder;
pub struct AlchemySubscriptionBuilder;

impl SubscriptionBuilder for PolygonSubscriptionBuilder {
    fn build_subscription_messages(params: &[(&str, Vec<String>)]) -> Vec<Message> {
        params.iter().map(|&(param, ref topics)| {
            topics.iter().map(|topic| {
                Message::Text(format!(r#"{{"action":"subscribe","params":"{}@{}"}}"#, topic, param))
            }).collect::<Vec<_>>()
        }).flatten().collect()
    }
}

impl SubscriptionBuilder for BinanceSubscriptionBuilder {
    fn build_subscription_messages(params: &[(&str, Vec<String>)]) -> Vec<Message> {
        params.iter().flat_map(|&(base, ref topics)| {
            let params: Vec<String> = topics.iter().map(|topic| format!("{}@{}", topic, base)).collect();
            vec![Message::Text(serde_json::to_string(&json!({
                "method": "SUBSCRIBE",
                "params": params,
                "id": 1
            })).unwrap())]
        }).collect()
    }
}

impl SubscriptionBuilder for AlchemySubscriptionBuilder {
    fn build_subscription_messages(params: &[(&str, Vec<String>)]) -> Vec<Message> {
        params.iter().map(|(chain, topics)| {
            topics.iter().map(|topic| {
                Message::Text(
                    serde_json::to_string(&json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "eth_subscribe",
                        "params": [topic],
                    })).expect("Failed to serialize subscription message")
                )
            }).collect::<Vec<_>>()
        }).flatten().collect()
    }
}
