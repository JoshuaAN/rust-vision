use std::error::Error;

use shared::SharedFrame;

pub trait Camera {
  fn new(index: u32, width: u32, height: u32) -> Result<Self, Box<dyn Error>>
  where 
        Self: Sized;

  fn grab_frame(&mut self) -> Result<SharedFrame, Box<dyn Error>>;
}