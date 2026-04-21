use std::thread;
use std::time::Instant; 

use image::Rgb;
use imageproc::drawing::{draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut};
use imageproc::rect::Rect;
use rusttype::{Font, Scale};
use rayon::prelude::*;

use video_capture::{Camera, CameraNokhwa}; 
use vision_apriltag::AprilTagDetector;
use vision_object::{ObjectDetector, ObjectDetectorOnnx};

use video_encode::{VideoEncoder, X264Encoder};
use video_stream::WebRtcBroadcaster;

use axum::{Router, extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}}, routing::{get, post}};
use tower_http::cors::{Any, CorsLayer};

use vision_core::{ApriltagDetection, EncodedFrame, PipelineResult, SharedFrame};
use tokio::sync::watch;

#[derive(Clone)]
struct AppState {
    broadcaster: WebRtcBroadcaster,
    meta_rx: watch::Receiver<PipelineResult>,
}

#[tokio::main]
async fn main() {
    let width = 640.0;
    let height = 480.0;

    let (frame_tx, mut frame_rx) = watch::channel::<Option<SharedFrame>>(None);
    let (drawn_frame_tx, mut drawn_frame_rx) = watch::channel::<Option<SharedFrame>>(None);
    let (meta_tx, meta_rx) = watch::channel(PipelineResult::default());

    // ---------------------------------------------------------
    // 1. HARDWARE CAPTURE THREAD
    // ---------------------------------------------------------
    thread::spawn(move || {
        let mut cam = CameraNokhwa::new(0, width as u32, height as u32).expect("Failed to init camera hardware");
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

    // ---------------------------------------------------------
    // 2. PARALLEL PROCESSING & DRAWING THREAD
    // ---------------------------------------------------------
    let drawn_tx_clone = drawn_frame_tx.clone();
    let rt = tokio::runtime::Handle::current();

    // Run this in a standard OS thread since CV operations are CPU-blocking
    thread::spawn(move || {
        let mut tag_detector = AprilTagDetector::new();
        let mut obj_detector = ObjectDetectorOnnx::new("yolo11n.onnx").expect("Failed to init object detector");

        let cyan = Rgb([0, 242, 255]);
        let neon_green = Rgb([57, 255, 20]);
        
        // Load font for drawing text (Ensure this file exists or change the path to a valid .ttf on your system)
        let font_data: &[u8] = include_bytes!("../font.ttf");
        let font = Font::try_from_bytes(font_data).expect("Error constructing Font");
        let font_scale = Scale { x: 16.0, y: 16.0 };

        loop {
            // Wait for a fresh frame from the camera using Tokio's block_on
            if rt.block_on(frame_rx.changed()).is_err() { break; }
            
            if let Some(frame) = frame_rx.borrow().clone() {
                
                // --- PARALLEL EXECUTION ---
                // rayon::join splits this workload across two threads and waits for both to finish.
                // We wrap the object detection in a block to measure its exact duration.
                let (tags, (objects, obj_latency_ms)) = rayon::join(
                    || tag_detector.detect(&frame),
                    || {
                        let start_time = Instant::now();
                        let det = obj_detector.detect(&frame);
                        let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                        (det, elapsed_ms as f32) // Assuming latency_ms is f32; remove "as f32" if it is f64
                    }
                );

                println!("Object inference time: {obj_latency_ms}ms");

                // Send JSON metadata to WebSocket (optional, just in case you need it for UI data)
                let _ = meta_tx.send(PipelineResult {
                    frame_timestamp: frame.timestamp_ms,
                    tags: tags.iter().map(|f| { ApriltagDetection { id: f.id, corners: f.corners } }).collect(),
                    latency_ms: obj_latency_ms as f64, // Passed the timed duration here
                    objects: objects.clone(),
                });

                // --- SYNCHRONOUS DRAWING ---
                let mut annotated_frame = frame.clone();
                
                let raw_bytes: Vec<u8> = (*annotated_frame.data).clone();
                
                // 2. Wrap our mutable Vec<u8> into an ImageBuffer
                if let Some(mut img_buf) = image::ImageBuffer::<Rgb<u8>, _>::from_raw(
                    width as u32, 
                    height as u32, 
                    raw_bytes
                ) {
                    // Draw Objects (YOLO)
                    for obj in &objects {
                        let [x1, y1, x2, y2] = obj.box_2d;
                        let rect = Rect::at(x1 as i32, y1 as i32).of_size((x2 - x1) as u32, (y2 - y1) as u32);
                        
                        draw_hollow_rect_mut(&mut img_buf, rect, cyan);
                        
                        // Added latency to the YOLO label drawing for real-time visual feedback
                        let label = format!("{} {:.0}% ({:.1}ms)", obj.label, obj.confidence * 100.0, obj_latency_ms);
                        draw_text_mut(&mut img_buf, cyan, x1 as i32, (y1 - 20.0) as i32, font_scale, &font, &label);
                    }

                    // Draw AprilTags
                    for tag in &tags {
                        let c = tag.corners;
                        for i in 0..4 {
                            let start = c[i];
                            let end = c[(i + 1) % 4];
                            
                            draw_line_segment_mut(
                                &mut img_buf, 
                                (start.0 as f32, start.1 as f32), 
                                (end.0 as f32, end.1 as f32), 
                                neon_green
                            );
                        }
                        let id_label = format!("ID: {}", tag.id);
                        draw_text_mut(&mut img_buf, neon_green, c[0].0 as i32, (c[0].1 - 20.0) as i32, font_scale, &font, &id_label);
                    }

                    // Extract the modified raw bytes back into your struct 
                    annotated_frame.data = std::sync::Arc::new(img_buf.into_raw());
                } else {
                    eprintln!("Warning: Could not construct ImageBuffer. The width/height do not match the byte length of annotated_frame.data.");
                }

                // Send the finished, drawn frame to the H.264 encoder
                let _ = drawn_tx_clone.send(Some(annotated_frame));            
            }
        }
    });

    // ---------------------------------------------------------
    // 3. ENCODER THREAD
    // ---------------------------------------------------------
    let (h264_tx, h264_rx) = watch::channel(EncodedFrame::default());
    let rt2 = tokio::runtime::Handle::current();
    let mut encoder_rx = drawn_frame_rx.clone();
    let mut x264 = X264Encoder::new(width as u32, height as u32).expect("Failed to create x264 encoder");

    thread::spawn(move || {
        let mut last_print = Instant::now();
        let mut frames_this_second = 0;
        let mut bytes_this_second = 0;

        loop {
            if rt2.block_on(encoder_rx.changed()).is_err() {
                break; 
            }

            if let Some(frame) = encoder_rx.borrow().clone() {
                match x264.encode(&frame) {
                    Ok(payload) => {
                        if !payload.data.is_empty() {
                            let _ = h264_tx.send(payload.clone());
                            
                            bytes_this_second += payload.data.len();
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

    // ---------------------------------------------------------
    // 4. WEBRTC BROADCAST & API SERVER
    // ---------------------------------------------------------
    let broadcaster = WebRtcBroadcaster::start(h264_rx);

    let shared_state = AppState {
        broadcaster,
        meta_rx,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/webrtc/sdp", post(sdp_handler))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Signaling Server listening on http://0.0.0.0:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn sdp_handler(State(state): State<AppState>, body: String) -> String {
    match state.broadcaster.handle_sdp_offer(&body).await {
        Ok(answer) => answer,
        Err(e) => {
            eprintln!("Failed to handle SDP: {}", e);
            String::from("Error creating WebRTC connection")
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state.meta_rx))
}

async fn handle_socket(mut socket: WebSocket, mut meta_rx: watch::Receiver<PipelineResult>) {
    while meta_rx.changed().await.is_ok() {
        let result = meta_rx.borrow().clone();
        if let Ok(json) = serde_json::to_string(&result) {
            if socket.send(Message::Text(json)).await.is_err() {
                break; 
            }
        }
    }
}