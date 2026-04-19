use std::collections::BTreeMap;

use camera::CameraNode;
use streamer::{StreamerNode, web_server};
use vision::VisionNode; // Assuming you have your vision crate ready
use shared::{SharedFrame, PipelineResult}; // Use your renamed shared crate
use minifb::{Key, Window, WindowOptions};
use image::{RgbImage, Rgb};
use imageproc::drawing::draw_line_segment_mut;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let width = 640;
    let height = 480;
    
    // 1. Start Camera
    let cam_node = CameraNode::start(0, width, height);
    let mut frame_rx = cam_node.frame_rx.clone();

    // 2. Start Vision
    let vision_node = VisionNode::start(cam_node.frame_rx.clone());
    let mut results_rx = vision_node.results_rx.clone();

    // 3. Create a channel for Annotated Frames (RGB)
    let (annotated_tx, mut annotated_rx) = watch::channel(None);

    let mut frame_buffer: BTreeMap<u128, SharedFrame> = BTreeMap::new();

tokio::spawn(async move {
    loop {
        // 1. Listen for BOTH frames and results
        tokio::select! {
            // New frame arrives from camera
            Ok(_) = frame_rx.changed() => {
                if let Some(frame) = frame_rx.borrow().clone() {
                    // Store frame by timestamp
                    frame_buffer.insert(frame.timestamp_ms as u128, frame);
                    
                    // Prevent memory leaks: Keep only the last 10 frames (~300ms buffer)
                    if frame_buffer.len() > 10 {
                        let first_key = *frame_buffer.keys().next().unwrap();
                        frame_buffer.remove(&first_key);
                    }
                }
            }

            // New vision result arrives
            Ok(_) = results_rx.changed() => {
                if let Some(res) = results_rx.borrow().clone() {
                    // 2. Find the EXACT frame this math belongs to
                    if let Some(original_frame) = frame_buffer.get(&(res.frame_timestamp as u128)) {
                        let mut data = original_frame.data.to_vec();
                        
                        for tag in res.tags {
                            draw_tag_outline(original_frame.width, original_frame.height, &mut data, &tag.corners);
                        }

                        let annotated = SharedFrame {
                            width: original_frame.width,
                            height: original_frame.height,
                            timestamp_ms: original_frame.timestamp_ms,
                            data: std::sync::Arc::new(data),
                        };

                        // 3. Send to stream/window
                        let _ = annotated_tx.send(Some(annotated));
                    }
                }
            }
        }
    }
});

    // 5. Start Streamer (now using the annotated receiver!)
    let streamer_rx = annotated_rx.clone();
    tokio::spawn(async move {
        StreamerNode::start(streamer_rx, width, height, 5800).await;
    });

    tokio::spawn(async move {
        web_server::start_web_server(8080).await;
    });

    // 6. Local Debug Window
    let mut window = Window::new("Local Feed", width as usize, height as usize, WindowOptions::default()).unwrap();
    let mut fb_buffer: Vec<u32> = vec![0; (width * height) as usize];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if annotated_rx.changed().await.is_ok() {
            if let Some(frame) = annotated_rx.borrow().clone() {
                // Convert RGB to u32 for minifb
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

fn draw_tag_outline(width: u32, height: u32, data: &mut [u8], corners: &[(f64, f64); 4]) {
    // Wrap our unique copy in an ImageBuffer
    // The 'data' here is now a &mut [u8], so this is allowed!
    if let Some(mut img) = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, data) {
        let color = Rgb([0, 255, 0]); 

        for i in 0..4 {
            let start = corners[i];
            let end = corners[(i + 1) % 4];
            draw_line_segment_mut(
                &mut img, 
                (start.0 as f32, start.1 as f32), 
                (end.0 as f32, end.1 as f32), 
                color
            );
        }
    }
}