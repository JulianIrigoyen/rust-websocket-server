// use futures_util::SinkExt;
// use serde_json::json;
// use tokio::net::TcpStream;
// use tokio_tungstenite::{connect_async, MaybeTlsStream, tungstenite::protocol::Message, WebSocketStream};
// use url::Url;
// use crate::subscribers::websocket_subscriber::WebSocketSubscriber;
//
// pub struct BinanceSubscriber {
//     ws_url: String,
// }
//
// #[async_trait]
// impl WebSocketSubscriber for BinanceSubscriber {
//     async fn connect(&self) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn std::error::Error>> {
//         let url = Url::parse(&self.ws_url)?;
//         let (ws_stream, _) = connect_async(url).await?;
//         println!("Connected to Binance WebSocket");
//         Ok(ws_stream)
//     }
//
//     async fn subscribe(&self, ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
//         let subscriptions = vec![
//             json!({"method": "SUBSCRIBE", "params": ["btcusdt@aggTrade"], "id": 1}),
//             // Add other subscriptions as needed.
//         ];
//
//         for subscription in subscriptions {
//             ws_stream.send(Message::Text(subscription.to_string())).await?;
//         }
//         println!("Subscribed to Binance channels");
//         Ok(())
//     }
//
//     async fn consume_messages(&self, ws_stream: &WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<(), Box<dyn std::error::Error>> {
//         let mut stream = ws_stream.clone();
//         while let Some(message) = stream.next().await {
//             match message? {
//                 Message::Text(text) => {
//                     println!("Received message: {}", text);
//                     // Process the message as needed.
//                 }
//                 _ => (),
//             }
//         }
//         Ok(())
//     }
// }
//
// impl BinanceSubscriber {
//     pub fn new(ws_url: &str) -> Self {
//         Self {
//             ws_url: ws_url.to_string(),
//         }
//     }
// }