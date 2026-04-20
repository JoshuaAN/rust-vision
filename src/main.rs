use std::{collections::BTreeMap};
use std::fs;

use rusttype::{Font, Scale};
use camera::CameraNode;
use streamer::{StreamerNode, web_server};
use vision_apriltag::ApriltagNode; // Assuming you have your vision crate ready
use shared::{ObjectDetection, PipelineResult, SharedFrame}; // Use your renamed shared crate
use minifb::{Key, Window, WindowOptions};
use image::{DynamicImage, Rgb, RgbImage};
use imageproc::{drawing::{draw_filled_rect_mut, draw_hollow_rect_mut, draw_line_segment_mut, draw_text_mut}, rect::Rect};
use tokio::sync::watch;
use vision_object::ObjectNode;

#[tokio::main]
async fn main() {
    let width = 640;
    let height = 480;

let font_data = std::fs::read("font.ttf").expect("Failed to read font.ttf");
    let font = Font::try_from_vec(font_data).unwrap();

    // 1. Start Camera
    let cam_node = CameraNode::start(0, width, height);
    let mut frame_rx = cam_node.frame_rx.clone();

    // 2. Start Vision
    let vision_node = ApriltagNode::start(cam_node.frame_rx.clone());
    let mut results_rx = vision_node.results_rx.clone();

    let mut ai_detector = ObjectNode::new("yolo11n.onnx");

    // 3. Create a channel for Annotated Frames (RGB)
    let (annotated_tx, mut annotated_rx) = watch::channel(None);

    let mut frame_buffer: BTreeMap<u128, SharedFrame> = BTreeMap::new();

tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle Camera Frames
                Ok(_) = frame_rx.changed() => {
                    if let Some(frame) = frame_rx.borrow().clone() {
                        frame_buffer.insert(frame.timestamp_ms as u128, frame);
                        if frame_buffer.len() > 15 { // Slightly larger buffer for AI latency
                            let first_key = *frame_buffer.keys().next().unwrap();
                            frame_buffer.remove(&first_key);
                        }
                    }
                }

                // Handle Combined Vision Processing
                Ok(_) = results_rx.changed() => {
                    if let Some(res) = results_rx.borrow().clone() {
                        if let Some(original_frame) = frame_buffer.get(&(res.frame_timestamp as u128)) {
                            let mut data = original_frame.data.to_vec();
                            
                            // A. Draw AprilTags
                            for tag in &res.tags {
                                draw_tag_outline(original_frame.width, original_frame.height, &mut data, &tag.corners);
                            }

                            // B. Run AI Inference on this specific frame
                            // We convert the raw buffer to a DynamicImage for the detector
                            let img_buffer = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(
                                original_frame.width, original_frame.height, data.clone()
                            ).unwrap();
                            let dyn_img = DynamicImage::ImageRgb8(img_buffer);
                            
                            let detections = ai_detector.detect(&dyn_img);

                            // C. Draw AI Bounding Boxes
                            for obj in detections {
                                // PASS THE FONT REFERENCE HERE
                                draw_ai_detection(original_frame.width, original_frame.height, &mut data, &obj, &font);
                            }

                            let annotated = SharedFrame {
                                width: original_frame.width,
                                height: original_frame.height,
                                timestamp_ms: original_frame.timestamp_ms,
                                data: std::sync::Arc::new(data),
                            };

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

// Drawing helper for AI Boxes and Labels
// Requires: use imageproc::drawing::{draw_hollow_rect_mut, draw_filled_rect_mut, draw_text_mut};
fn draw_ai_detection(
    width: u32, 
    height: u32, 
    data: &mut [u8], 
    obj: &shared::ObjectDetection, 
    font: &rusttype::Font<'_>
) {
    if let Some(mut img) = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, data) {
        let ai_color = Rgb([255, 0, 0]); // Red for AI
        let text_color = Rgb([255, 255, 255]); // White for Text
        
        let bbox = obj.box_2d; // [x_min, y_min, x_max, y_max]
        let x_min = bbox[0] as i32;
        let y_min = bbox[1] as i32;
        
        // 1. Draw the Bounding Box
        let rect = Rect::at(x_min, y_min)
            .of_size((bbox[2] - bbox[0]) as u32, (bbox[3] - bbox[1]) as u32);
        draw_hollow_rect_mut(&mut img, rect, ai_color);

        // 2. Prepare the Text Label
        // Convert confidence 0.854 to "85%"
        let conf_percent = (obj.confidence * 100.0) as u32;
        let label_text = format!("{} {}%", obj.label, conf_percent);
        
        // Scale and size are standard for 640x480
        let scale = Scale { x: 16.0, y: 16.0 };
        let text_height = 18;

        // 3. Draw Text Background Box (to ensure readability)
        // Position it just above the top of the main box
        let background_rect = Rect::at(x_min, y_min - text_height)
            .of_size((label_text.len() * 9) as u32, text_height as u32); // Estimate width
            
        // Basic clamp to prevent drawing outside image boundaries
        if y_min - text_height > 0 {
             draw_filled_rect_mut(&mut img, background_rect, ai_color);
             
             // 4. Draw the Label Text
             draw_text_mut(
                &mut img, 
                text_color, 
                x_min + 2, // Small padding
                y_min - text_height + 1, // small padding
                scale, 
                font, 
                &label_text
            );
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