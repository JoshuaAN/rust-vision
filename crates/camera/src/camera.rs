use std::error::Error;

use vision_core::SharedFrame;

/// An abstraction around a physical camera in hardware which frames can be captured from.
pub trait Camera {
  fn grab_frame(&mut self) -> Result<SharedFrame, Box<dyn Error>>;
}