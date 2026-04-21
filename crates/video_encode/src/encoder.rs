use std::error::Error;

use vision_core::{SharedFrame, EncodedFrame};

pub trait VideoEncoder: Send + 'static {
  fn encode(&mut self, frame: &SharedFrame) -> Result<EncodedFrame, Box<dyn Error>>;
}