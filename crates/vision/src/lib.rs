use shared::{SharedFrame};
use std::thread;
use tokio::runtime::Handle;
use tokio::sync::watch;

// IMPORT YOUR CUSTOM CRATE HERE
use apriltag::{Detector, ApriltagDetection, DetectorConfig}; 

#[derive(Clone)]
pub struct PipelineResult {
    pub frame_timestamp: u64,
    pub latency_ms: f64,
    pub tags: Vec<ApriltagDetection>,
}

pub struct VisionNode {
    pub results_rx: watch::Receiver<Option<PipelineResult>>,
}

impl VisionNode {
    pub fn start(mut frame_rx: watch::Receiver<Option<SharedFrame>>) -> Self {
        // Channel to broadcast the math/targeting results
        let (tx, rx) = watch::channel(None);
        let rt = Handle::current();

        thread::spawn(move || {
            // 1. Initialize your custom AprilTag wrapper here
            // let mut detector = Detector::new(DetectorConfig::default());
            println!("Vision thread initialized.");

            loop {
                // Wait for a new frame from the camera
                if rt.block_on(frame_rx.changed()).is_err() {
                    println!("Camera stream closed, stopping vision thread.");
                    break;
                }

                if let Some(frame) = frame_rx.borrow().clone() {
                    let start_time = std::time::Instant::now();

                    // 2. Convert RGB to Grayscale
                    let gray_image = rgb_to_grayscale(frame.width as usize, frame.height as usize, &frame.data);

                    // 3. Run your custom detector
                    // Let's assume your crate takes a flat &[u8] slice, width, and height
                    // let raw_detections = detector.detect(&gray_image, frame.width, frame.height);
                    
                    // --- MOCK DETECTIONS FOR NOW ---
                    let raw_detections = vec![
                        ApriltagDetection { id: 4, corners: [(0.0, 0.0); 4] } // Fake tag dead center
                    ];
                    // -------------------------------

                    // 4. Map the results into our shared `PipelineResult` struct
                    let mut tags = Vec::new();
                    for d in raw_detections {
                        tags.push(ApriltagDetection { id: 4, corners: [(0.0, 0.0); 4] });
                    }

                    // Calculate how long detection took (critical for FRC latency compensation)
                    let latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;

                    // let result = Pi {
                    //     frame_timestamp: frame.timestamp_ms,
                    //     latency_ms,
                    //     tags,
                    // };

                    // // Broadcast the result to telemetry, streamer, etc.
                    // let _ = tx.send(Some(result));
                }
            }
        });

        Self { results_rx: rx }
    }
}

/// Fast, rough RGB to Grayscale conversion
/// We use a fast integer approximation instead of floating point math for speed.
fn rgb_to_grayscale(width: usize, height: usize, rgb: &[u8]) -> Vec<u8> {
    let mut gray = vec![0u8; width * height];
    
    // Process pixels in chunks of 3 (R, G, B)
    for (i, chunk) in rgb.chunks_exact(3).enumerate() {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        
        // Fast luminance approximation: (R + 2G + B) / 4
        // Bit-shifting (>> 2) is incredibly fast on a CPU
        gray[i] = ((r + (g << 1) + b) >> 2) as u8; 
    }
    
    gray
}