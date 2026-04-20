use std::error::Error;

use shared::{ObjectDetection, SharedFrame};

/// Represents an object detector pipeline.
pub trait ObjectDetector {
  fn new(model_path: &str) -> Result<Self, Box<dyn Error>> where Self: Sized;

  fn detect(&mut self, frame: &SharedFrame) -> Vec<ObjectDetection>;
}