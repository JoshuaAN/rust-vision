mod frame;

pub use frame::{SharedFrame, EncodedFrame};

#[derive(Clone, serde::Serialize)]
pub struct ApriltagDetection {
    pub id: u32,
    pub corners: [(f64, f64); 4],
}

#[derive(Clone, serde::Serialize)]
pub struct ObjectDetection {
    pub label: String,
    pub confidence: f32,
    pub box_2d: [f32; 4],
}

#[derive(Clone, serde::Serialize)]
pub struct PipelineResult {
    pub frame_timestamp: u64,
    pub latency_ms: f64,
    pub tags: Vec<ApriltagDetection>,
    pub objects: Vec<ObjectDetection>,
}

impl Default for PipelineResult {
    fn default() -> Self {
        Self { frame_timestamp: 0, latency_ms: 0.0, tags: Vec::new(), objects: Vec::new() }
    }
}
