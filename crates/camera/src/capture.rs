use shared::SharedFrame;
use nokhwa::{Camera, pixel_format::RgbFormat, utils::{CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution}};
use std::sync::Arc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

pub struct CameraNode {
    pub frame_rx: watch::Receiver<Option<SharedFrame>>,
}

impl CameraNode {
pub fn start(index: u32, width: u32, height: u32) -> Self {
        // Create a watch channel holding "None" initially
        let (tx, rx) = watch::channel(None);

        // Spawn a dedicated hardware thread
        thread::spawn(move || {
            let cam_index = CameraIndex::Index(index);
            
            // 1. Configure the requested camera format
            let target_format = CameraFormat::new(
                Resolution::new(width, height), 
                FrameFormat::YUYV, 
                30 // Target FPS
            );
            
            let requested = RequestedFormat::new::<RgbFormat>(
                RequestedFormatType::Closest(target_format)
            );

            // 2. Initialize and open the camera stream
            let mut camera = Camera::new(cam_index, requested).expect("Failed to initialize camera");
            camera.open_stream().expect("Failed to open camera stream");

            println!("Camera {} started successfully at {}x{}", index, width, height);

            // 3. Continuous capture loop
            loop {
                match camera.frame() {
                    Ok(frame) => {
                        // Decode the frame into RGB
                        if let Ok(decoded) = frame.decode_image::<RgbFormat>() {
                            
                            // .into_raw() extracts the Vec<u8> from the ImageBuffer directly (no clone)
                            let frame_data = decoded.into_raw();

                            let shared = SharedFrame {
                                timestamp_ms: SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64,
                                width,
                                height,
                                data: Arc::new(frame_data), 
                            };

                            // Push the newest frame to anyone listening
                            // If the main thread drops the receiver, this errors and we cleanly exit the loop
                            if tx.send(Some(shared)).is_err() {
                                println!("All camera receivers dropped. Shutting down capture thread.");
                                break; 
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Camera capture error: {:?}", e);
                    }
                }
            }
        });

        Self { frame_rx: rx }
    }
}