use vision_core::SharedFrame;
use apriltag::{ApriltagDetection, Detector, DetectorConfig, ImageChannel};

pub struct AprilTagDetector {
    detector: Detector,
}

impl AprilTagDetector {
    pub fn new() -> Self {
        let detector = Detector::new(apriltag::Backend::UMich, DetectorConfig::default());
        Self { detector }
    }

    pub fn detect(&mut self, frame: &SharedFrame) -> Vec<ApriltagDetection> {
        let gray_image = rgb_to_grayscale(frame.width as usize, frame.height as usize, &frame.data);

        let raw_detections = self.detector.detect(
            &ImageChannel {
                data: gray_image,
                width: frame.width as usize,
                height: frame.height as usize,
            },
            5, 
        );

        raw_detections.into_iter().collect()
    }
}

// Fast bitwise RGB to Grayscale conversion
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