use shared::{ObjectDetection, SharedFrame};

/// Represents an object detector pipeline.
pub trait ObjectDetector {
  fn detect(&mut self, frame: &SharedFrame) -> Vec<ObjectDetection>;
}