use shared::{PipelineResult, SharedFrame};
use std::thread;
use tokio::runtime::Handle;
use tokio::sync::watch;

use apriltag::{ApriltagDetection, Detector, DetectorConfig, ImageChannel}; 

pub struct ApriltagNode {
    pub results_rx: watch::Receiver<Option<PipelineResult>>,
}

impl ApriltagNode {
    pub fn start(mut frame_rx: watch::Receiver<Option<SharedFrame>>) -> Self {
        let (tx, rx) = watch::channel(None);
        let rt = Handle::current();

        thread::spawn(move || {
            let mut detector = Detector::new(apriltag::Backend::UMich, DetectorConfig::default());
            println!("Vision thread initialized.");

            loop {
                if rt.block_on(frame_rx.changed()).is_err() {
                    println!("Camera stream closed, stopping vision thread.");
                    break;
                }

                if let Some(frame) = frame_rx.borrow().clone() {
                    let start_time = std::time::Instant::now();

                    let gray_image = rgb_to_grayscale(frame.width as usize, frame.height as usize, &frame.data);

                    let raw_detections = detector.detect(&ImageChannel { data: gray_image, width: frame.width as usize, height: frame.height as usize}, 5);

                    let mut tags = Vec::new();
                    for d in raw_detections {
                        tags.push(d);
                    }

                    let latency_ms = start_time.elapsed().as_secs_f64() * 1000.0;

                    let result = PipelineResult {
                        frame_timestamp: frame.timestamp_ms,
                        latency_ms,
                        tags,
                        objects: Vec::new(),
                    };

                    let _ = tx.send(Some(result));
                }
            }
        });

        Self { results_rx: rx }
    }
}

fn rgb_to_grayscale(width: usize, height: usize, rgb: &[u8]) -> Vec<u8> {
    let mut gray = vec![0u8; width * height];
    
    for (i, chunk) in rgb.chunks_exact(3).enumerate() {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        
        gray[i] = ((r + (g << 1) + b) >> 2) as u8; 
    }
    
    gray
}