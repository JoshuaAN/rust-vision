use std::thread;

use axum::{
    extract::{State, WebSocketUpgrade},
    extract::ws::{Message, WebSocket},
    response::Html,
    routing::get,
    Router,
};
use shared::SharedFrame;
use tokio::{net::TcpListener, runtime::Handle};
use tokio::sync::watch;
use x264::{Colorspace, Image, Plane, Preset, Setup, Tune};

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
    /// Starts the H.264 encoder and Axum web server.
    /// This function blocks indefinitely while serving the web application.
    pub async fn start(
        mut frame_rx: watch::Receiver<Option<SharedFrame>>,
        width: u32,
        height: u32,
        port: u16,
    ) {
        // Channel to pass encoded H.264 packets from the encoder to the WebSocket connections
        let (h264_tx, h264_rx) = watch::channel(Vec::new());

        let rt = Handle::current();

        // 1. Dedicated Async Task for Encoding
        thread::spawn(move || {
            let mut encoder = Setup::preset(Preset::Ultrafast, Tune::None, false, true)
                .fps(30, 1)
                .build(Colorspace::I420, width as i32, height as i32)
                .expect("Failed to build x264 encoder");

            let mut pts = 0i64;

            loop {
                // block_on safely halts this specific OS thread until a frame arrives, 
                // without blocking the async web server!
                if rt.block_on(frame_rx.changed()).is_err() {
                    println!("Camera stream closed, stopping encoder.");
                    break; 
                }

                if let Some(frame) = frame_rx.borrow().clone() {
                    let (y, u, v) = rgb_to_i420(width as usize, height as usize, &frame.data);
                    
                    let plane_y = Plane { stride: width as i32, data: &y };
                    let plane_u = Plane { stride: (width / 2) as i32, data: &u };
                    let plane_v = Plane { stride: (width / 2) as i32, data: &v };

                    let x264_image = Image::new(
                        Colorspace::I420,
                        width as i32,
                        height as i32,
                        &[plane_y, plane_u, plane_v]
                    );

                    match encoder.encode(pts, x264_image) {
                        Ok((data, _picture)) => {
                            let frame_payload = data.entirety().to_vec();
                            if !frame_payload.is_empty() {
                                // watch::Sender::send is synchronous, so it works perfectly here
                                let _ = h264_tx.send(frame_payload);
                            }
                            pts += 1;
                        }
                        Err(e) => eprintln!("x264 Encoding Error: {:?}", e),
                    }
                }
            }
        });

        // 2. Setup and run the Web Server
        let app = Router::new()
            .route("/", get(|| async { Html(INDEX_HTML) }))
            .route("/ws", get(ws_handler))
            .with_state(h264_rx);

        let addr = format!("0.0.0.0:{}", port);
        println!("Starting streaming server on http://{}", addr);
        
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind to port");
        axum::serve(listener, app).await.expect("Failed to start Axum server");
    }
}

/// Convert Interleaved RGB8 to Planar I420 (YUV420p)
fn rgb_to_i420(width: usize, height: usize, rgb: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut y_plane = vec![0u8; width * height];
    let mut u_plane = vec![0u8; (width / 2) * (height / 2)];
    let mut v_plane = vec![0u8; (width / 2) * (height / 2)];

    for j in 0..height {
        for i in 0..width {
            let idx = (j * width + i) * 3;
            let r = rgb[idx] as f32;
            let g = rgb[idx + 1] as f32;
            let b = rgb[idx + 2] as f32;

            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            y_plane[j * width + i] = y.clamp(0.0, 255.0) as u8;

            if j % 2 == 0 && i % 2 == 0 {
                let u = -0.1687 * r - 0.3313 * g + 0.5 * b + 128.0;
                let v = 0.5 * r - 0.4187 * g - 0.0813 * b + 128.0;
                let uv_idx = (j / 2) * (width / 2) + (i / 2);
                u_plane[uv_idx] = u.clamp(0.0, 255.0) as u8;
                v_plane[uv_idx] = v.clamp(0.0, 255.0) as u8;
            }
        }
    }
    (y_plane, u_plane, v_plane)
}

async fn ws_handler(ws: WebSocketUpgrade, State(rx): State<watch::Receiver<Vec<u8>>>) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, rx))
}

async fn handle_socket(mut socket: WebSocket, mut rx: watch::Receiver<Vec<u8>>) {
    loop {
        if rx.changed().await.is_ok() {
            let frame = rx.borrow().clone();
            if frame.is_empty() { continue; }
            
            if socket.send(Message::Binary(frame)).await.is_err() {
                break; // Client disconnected
            }
        } else {
            break; // Server channel closed
        }
    }
}