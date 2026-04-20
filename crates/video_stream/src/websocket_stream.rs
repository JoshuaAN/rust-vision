use axum::extract::ws::{Message, WebSocket};
use tokio::sync::watch;

/// Manages WebSocket connections and streams H.264 binary frames.
#[derive(Clone)]
pub struct WebSocketBroadcaster {
    h264_rx: watch::Receiver<Vec<u8>>,
}

impl WebSocketBroadcaster {
    pub fn new(h264_rx: watch::Receiver<Vec<u8>>) -> Self {
        Self { h264_rx }
    }

    /// Takes an upgraded WebSocket connection and continuously feeds it video frames.
    pub async fn handle_client(mut self, mut socket: WebSocket) {
        println!("WebSocket client connected");
        
        loop {
            // Wait for a new frame
            if self.h264_rx.changed().await.is_err() {
                println!("Video feed closed, disconnecting WS client.");
                break;
            }

            let frame = self.h264_rx.borrow().clone();
            if frame.is_empty() {
                continue;
            }

            // Send the raw H.264 bytes
            if socket.send(Message::Binary(frame)).await.is_err() {
                println!("WebSocket client disconnected");
                break;
            }
        }
    }
}