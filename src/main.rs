use std::thread;
use std::time::Instant; 

use video_capture::{Camera, CameraNokhwa}; 
use vision_apriltag::AprilTagDetector;
use vision_object::{ObjectDetectorOnnx};

use video_encode::{VideoEncoder, X264Encoder};
use video_stream::WebRtcBroadcaster;

use axum::{routing::post, Router, extract::State};
use tower_http::cors::{Any, CorsLayer};

use image::Rgb;
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut},
    rect::Rect,
};
use rusttype::{Font, Scale};
use vision_core::{SharedFrame, ObjectDetection};
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let width = 640;
    let height = 480;

    let font_data = std::fs::read("font.ttf").expect("Failed to read font.ttf");
    let font = Font::try_from_vec(font_data).unwrap();

    let (frame_tx, mut frame_rx) = watch::channel::<Option<SharedFrame>>(None);

    // 1. Spawn Hardware Capture Thread
    thread::spawn(move || {
        let mut cam = CameraNokhwa::new(0, width, height).expect("Failed to init camera hardware");
        println!("Camera started successfully at {}x{}", width, height);

        loop {
            match cam.grab_frame() {
                Ok(frame) => {
                    if frame_tx.send(Some(frame)).is_err() {
                        break;
                    }
                }
                Err(e) => eprintln!("Camera capture error: {:?}", e),
            }
        }
    });

    let mut apriltag_detector = AprilTagDetector::new();
    let mut ai_detector = ObjectDetectorOnnx::new("yolo11n.onnx").expect("Failed to init object detector");

    let (annotated_tx, mut annotated_rx) = watch::channel(None);

    // 2. MAIN VISION ORCHESTRATION LOOP
    tokio::spawn(async move {
        let mut last_print = Instant::now();
        let mut frames_this_second = 0;

        loop {
            if frame_rx.changed().await.is_ok() {
                if let Some(frame) = frame_rx.borrow().clone() {
                    let mut data = frame.data.to_vec();
                    frames_this_second += 1;

                    if last_print.elapsed().as_secs() >= 1 {
                        println!("Vision Pipeline | {} FPS", frames_this_second);
                        frames_this_second = 0;
                        last_print = Instant::now();
                    }

                    // [Vision logic goes here]

                    let annotated = SharedFrame {
                        width: frame.width,
                        height: frame.height,
                        timestamp_ms: frame.timestamp_ms,
                        data: std::sync::Arc::new(data),
                    };

                    let _ = annotated_tx.send(Some(annotated));
                }
            } else {
                break; 
            }
        }
    });

    // 3. --- START VIDEO ENCODER THREAD ---
    let (h264_tx, h264_rx) = watch::channel(Vec::new());
    let rt = tokio::runtime::Handle::current();
    let mut encoder_rx = annotated_rx.clone();
    let mut x264 = X264Encoder::new(width, height).expect("Failed to create x264 encoder");

    thread::spawn(move || {
        let mut last_print = Instant::now();
        let mut frames_this_second = 0;
        let mut bytes_this_second = 0;

        loop {
            if rt.block_on(encoder_rx.changed()).is_err() {
                break; 
            }

            if let Some(frame) = encoder_rx.borrow().clone() {
                match x264.encode(&frame) {
                    Ok(payload) => {
                        if !payload.is_empty() {
                            let _ = h264_tx.send(payload.clone());
                            
                            bytes_this_second += payload.len();
                            frames_this_second += 1;

                            if last_print.elapsed().as_secs() >= 1 {
                                let kbps = (bytes_this_second as f64 * 8.0) / 1000.0;
                                println!("Video Encoder | {} FPS | Output: {:.2} kbps", frames_this_second, kbps);
                                frames_this_second = 0;
                                bytes_this_second = 0;
                                last_print = Instant::now();
                            }
                        }
                    }
                    Err(e) => eprintln!("Encoding error: {}", e),
                }
            }
        }
    });

    // 4. --- START WEBRTC BROADCASTER & HTTP SERVER ---
    // Start the WebRTC engine, passing it the H.264 stream
    let broadcaster = WebRtcBroadcaster::start(h264_rx);

    let _ = tokio::spawn(async move {
        // Allow the React dev server to hit this API
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/webrtc/sdp", post(sdp_handler))
            .layer(cors)
            .with_state(broadcaster); // Share the broadcaster with Axum

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
        println!("Signaling Server listening on http://0.0.0.0:8080/webrtc/sdp");
        axum::serve(listener, app).await.unwrap();
    }).await;
}

// Axum Handler for the SDP Handshake
async fn sdp_handler(
    State(broadcaster): State<WebRtcBroadcaster>,
    body: String,
) -> String {
    // We pass the offer to the WebRTC Engine, and return the answer back to the browser
    match broadcaster.handle_sdp_offer(&body).await {
        Ok(answer) => answer,
        Err(e) => {
            eprintln!("Failed to handle SDP: {}", e);
            String::from("Error creating WebRTC connection")
        }
    }
}

// ---------------------------------------------------------
// Drawing Helpers (Unchanged)
// ---------------------------------------------------------
fn draw_ai_detection(width: u32, height: u32, data: &mut [u8], obj: &ObjectDetection, font: &rusttype::Font<'_>) { /* ... */ }
fn draw_tag_outline(width: u32, height: u32, data: &mut [u8], corners: &[(f64, f64); 4]) { /* ... */ }