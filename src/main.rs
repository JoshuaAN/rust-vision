use std::thread;
use std::time::Instant; 

use camera::{Camera, CameraNokhwa}; 
use vision_apriltag::AprilTagDetector;
use vision_object::{ObjectDetector, ObjectDetectorOnnx};

use video_encode::{VideoEncoder, X264Encoder}; // Just pull the trait and struct
use streamer::{StreamerNode, web_server};

use image::{DynamicImage, Rgb};
use imageproc::{
    drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut},
    rect::Rect,
};
use minifb::{Key, Window, WindowOptions};
use rusttype::{Font, Scale};
use shared::{SharedFrame, ObjectDetection};
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let width = 640;
    let height = 480;

    // 1. Load Font Data
    let font_data = std::fs::read("font.ttf").expect("Failed to read font.ttf");
    let font = Font::try_from_vec(font_data).unwrap();

    // 2. Create the Camera Channel
    let (frame_tx, mut frame_rx) = watch::channel::<Option<SharedFrame>>(None);

    // 3. Spawn Hardware Capture Thread
    thread::spawn(move || {
        let mut cam = CameraNokhwa::new(0, width, height).expect("Failed to init camera hardware");
        println!("Camera started successfully at {}x{}", width, height);

        loop {
            match cam.grab_frame() {
                Ok(frame) => {
                    if frame_tx.send(Some(frame)).is_err() {
                        println!("Daemon shutting down camera thread.");
                        break;
                    }
                }
                Err(e) => eprintln!("Camera capture error: {:?}", e),
            }
        }
    });

    // 4. Initialize Vision Detectors
    let mut apriltag_detector = AprilTagDetector::new();
    let mut ai_detector = ObjectDetectorOnnx::new("yolo11n.onnx").expect("Failed to init object detector");

    // 5. Create a channel for Annotated Frames
    let (annotated_tx, mut annotated_rx) = watch::channel(None);

    // 6. MAIN VISION ORCHESTRATION LOOP
    tokio::spawn(async move {
        let mut last_print = Instant::now();
        let mut frames_this_second = 0;

        loop {
            if frame_rx.changed().await.is_ok() {
                if let Some(frame) = frame_rx.borrow().clone() {
                    let mut data = frame.data.to_vec();

                    frames_this_second += 1;

                    // Print Vision Pipeline FPS
                    if last_print.elapsed().as_secs() >= 1 {
                        println!("Vision Pipeline | {} FPS", frames_this_second);
                        frames_this_second = 0;
                        last_print = Instant::now();
                    }

                    // A. Run AprilTag Detection & Draw (Assumed skipped for brevity)
                    // B. Run AI Inference & Draw (Assumed skipped for brevity)

                    // C. Package the Annotated Frame and broadcast it
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

    // 7. --- START VIDEO ENCODER THREAD ---
    // Create the channel to pass compressed bytes to the streamer
    let (h264_tx, h264_rx) = watch::channel(Vec::new());
    
    // Get a handle to the Tokio runtime to block safely inside the std::thread
    let rt = tokio::runtime::Handle::current();
    let mut encoder_rx = annotated_rx.clone();

    // Instantiate the raw encoder
    let mut x264 = X264Encoder::new(width, height).expect("Failed to create x264 encoder");

    thread::spawn(move || {
        let mut last_print = Instant::now();
        let mut frames_this_second = 0;
        let mut bytes_this_second = 0;

        loop {
            // Safely wait for a new annotated frame to drop into the channel
            if rt.block_on(encoder_rx.changed()).is_err() {
                println!("Encoder thread shutting down (channel closed).");
                break; 
            }

            if let Some(frame) = encoder_rx.borrow().clone() {
                // Call your trait method
                match x264.encode(&frame) {
                    Ok(payload) => {
                        if !payload.is_empty() {
                            let _ = h264_tx.send(payload.clone());
                            
                            // True Bandwidth Tracking
                            bytes_this_second += payload.len();
                            frames_this_second += 1;

                            if last_print.elapsed().as_secs() >= 1 {
                                let kbps = (bytes_this_second as f64 * 8.0) / 1000.0;
                                println!(
                                    "Video Encoder | {} FPS | Output: {:.2} kbps",
                                    frames_this_second, kbps
                                );
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

    // 8. --- START STREAMER ---
    tokio::spawn(async move {
        // StreamerNode now cleanly takes the raw byte receiver
        StreamerNode::start(h264_rx, 5800).await;
    });

    // Start UI Web Server
    tokio::spawn(async move {
        web_server::start_web_server(8080).await;
    });

    // 9. Local Debug Window
    let mut window = Window::new(
        "Local Feed",
        width as usize,
        height as usize,
        WindowOptions::default(),
    ).unwrap();
    
    let mut fb_buffer: Vec<u32> = vec![0; (width * height) as usize];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if annotated_rx.changed().await.is_ok() {
            if let Some(frame) = annotated_rx.borrow().clone() {
                for i in 0..(width * height) as usize {
                    let r = frame.data[i * 3] as u32;
                    let g = frame.data[i * 3 + 1] as u32;
                    let b = frame.data[i * 3 + 2] as u32;
                    fb_buffer[i] = (r << 16) | (g << 8) | b;
                }
                window.update_with_buffer(&fb_buffer, width as usize, height as usize).unwrap();
            }
        }
    }
}

// ---------------------------------------------------------
// Drawing Helpers 
// ---------------------------------------------------------

fn draw_ai_detection(
    width: u32,
    height: u32,
    data: &mut [u8],
    obj: &ObjectDetection,
    font: &rusttype::Font<'_>,
) {
    if let Some(mut img) = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, data) {
        let ai_color = Rgb([255, 0, 0]); 
        let text_color = Rgb([255, 255, 255]); 

        let bbox = obj.box_2d; 
        let x_min = bbox[0] as i32;
        let y_min = bbox[1] as i32;

        let rect = Rect::at(x_min, y_min).of_size((bbox[2] - bbox[0]) as u32, (bbox[3] - bbox[1]) as u32);
        draw_hollow_rect_mut(&mut img, rect, ai_color);

        let conf_percent = (obj.confidence * 100.0) as u32;
        let label_text = format!("{} {}%", obj.label, conf_percent);

        let scale = Scale { x: 16.0, y: 16.0 };
        let text_height = 18;

        let background_rect = Rect::at(x_min, y_min - text_height)
            .of_size((label_text.len() * 9) as u32, text_height as u32); 

        if y_min - text_height > 0 {
            draw_filled_rect_mut(&mut img, background_rect, ai_color);
            draw_text_mut(
                &mut img,
                text_color,
                x_min + 2,               
                y_min - text_height + 1, 
                scale,
                font,
                &label_text,
            );
        }
    }
}

fn draw_tag_outline(width: u32, height: u32, data: &mut [u8], corners: &[(f64, f64); 4]) {
    if let Some(mut img) = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, data) {
        let color = Rgb([0, 255, 0]);

        for i in 0..4 {
            let start = corners[i];
            let end = corners[(i + 1) % 4];
            draw_line_segment_mut(
                &mut img,
                (start.0 as f32, start.1 as f32),
                (end.0 as f32, end.1 as f32),
                color,
            );
        }
    }
}