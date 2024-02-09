use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

pub struct WebSocketServer {
    client_senders: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>,
    host: String,
    port: String,
}

impl WebSocketServer {
    pub fn new(host: String, port: String) -> Self {
        WebSocketServer {
            client_senders: Arc::new(Mutex::new(Vec::new())),
            host,
            port,
        }
    }

    pub async fn run(&self) {
        let addr = format!("{}:{}", self.host, self.port);
        let try_socket = TcpListener::bind(&addr).await;
        let listener = try_socket.expect(&format!("Failed to bind to {}", addr));
        println!("WebSocket server listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let client_senders_clone = self.client_senders.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, client_senders_clone).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        stream: tokio::net::TcpStream,
        client_senders: Arc<Mutex<Vec<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = accept_async(stream).await?;

        {
            let mut senders = client_senders.lock().unwrap();
            senders.push(ws_stream);
        }

        // Might want to keep the connection alive, listen to messages from the client, etc.
        // For now, just adding the connection to the list

        Ok(())
    }
}
