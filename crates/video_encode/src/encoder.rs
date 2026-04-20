use std::error::Error;

use shared::SharedFrame;

pub trait VideoEncoder: Send + 'static {
  fn encode(&mut self, frame: &SharedFrame) -> Result<Vec<u8>, Box<dyn Error>>;
}