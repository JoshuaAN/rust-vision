use axum::{
    Router,
    extract::ws::{Message, WebSocket},
    extract::{State, WebSocketUpgrade},
    response::Html,
    routing::get,
};
use tokio::sync::watch;
use tokio::net::TcpListener;

// Assuming this exists in your project structure
pub mod web_server; 

const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Limelight H.264 Stream</title>
    <script type="text/javascript" src="https://cdn.jsdelivr.net/npm/jmuxer@2.0.5/dist/jmuxer.min.js"></script>
    <style>
        body { background: #111; color: white; display: flex; flex-direction: column; align-items: center; font-family: sans-serif; }
        video { width: 640px; height: 480px; background: #000; border: 2px solid #444; }
    </style>
</head>
<body>
    <h2>Robot Vision Stream</h2>
    <video id="player" autoplay muted controls></video>
    <script>
        window.onload = function() {
            var jmuxer = new JMuxer({
                node: 'player',
                mode: 'video',
                flushingTime: 0,
                fps: 30,
                debug: false
            });

            var ws = new WebSocket("ws://" + window.location.host + "/ws");
            ws.binaryType = 'arraybuffer';
            ws.onmessage = function(event) {
                jmuxer.feed({ video: new Uint8Array(event.data) });
            };
        };
    </script>
</body>
</html>
"#;

pub struct StreamerNode;

impl StreamerNode {
    /// Starts the Axum web server to stream H.264 packets over WebSockets.
    /// This function blocks indefinitely while serving the web application.
    pub async fn start(
        h264_rx: watch::Receiver<Vec<u8>>, // Accepts pre-encoded bytes!
        port: u16,
    ) {
        // Setup and run the Web Server, passing the receiver channel directly as State
        let app = Router::new()
            .route("/", get(|| async { Html(INDEX_HTML) }))
            .route("/ws", get(ws_handler))
            .with_state(h264_rx);

        let addr = format!("0.0.0.0:{}", port);
        println!("Starting streaming server on http://{}", addr);

        let listener = TcpListener::bind(&addr)
            .await
            .expect("Failed to bind to port");
            
        axum::serve(listener, app)
            .await
            .expect("Failed to start Axum server");
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(rx): State<watch::Receiver<Vec<u8>>>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: watch::Receiver<Vec<u8>>) {
    loop {
        // Wait for the encoder node to push a new frame payload
        if rx.changed().await.is_ok() {
            let frame = rx.borrow().clone();
            
            if frame.is_empty() {
                continue;
            }

            // Blast the bytes to the connected browser
            if socket.send(Message::Binary(frame)).await.is_err() {
                break; // Client disconnected
            }
        } else {
            break; // Server channel closed (Encoder died)
        }
    }
}