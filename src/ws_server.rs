use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;

async fn run_websocket_server() {
    let try_socket = TcpListener::bind("127.0.0.1:8080").await;
    let listener = try_socket.expect("Failed to bind");
    println!("WebSocket server listening on ws://127.0.0.1:8080");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: tokio::net::TcpStream) {
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("New WebSocket connection");

    // Split the WebSocket stream into a sender and receiver part
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Example of sending a message to the client
    // You would replace this with sending real data from your dataflow
    if let Err(e) = ws_sender.send(tungstenite::protocol::Message::Text("Hello WebSocket client!".to_string())).await {
        println!("Error sending message: {:?}", e);
    }

    // Listen for messages from the client, just as an example
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(msg) => println!("Received a message from the client: {:?}", msg),
            Err(e) => {
                println!("Error receiving message: {:?}", e);
                break;
            }
        }
    }
}
