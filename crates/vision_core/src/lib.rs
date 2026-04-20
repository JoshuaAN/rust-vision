mod frame;

use apriltag::ApriltagDetection;
pub use frame::SharedFrame;

#[derive(Clone)]
pub struct ObjectDetection {
    pub label: String,
    pub confidence: f32,
    pub box_2d: [f32; 4],
}

#[derive(Clone)]
pub struct PipelineResult {
    pub frame_timestamp: u64,
    pub latency_ms: f64,
    pub tags: Vec<ApriltagDetection>,
    pub objects: Vec<ObjectDetection>,
}
