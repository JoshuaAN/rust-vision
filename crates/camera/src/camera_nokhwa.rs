use std::{
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use nokhwa::{
    Camera as NokhwaCamera,
    pixel_format::RgbFormat,
    utils::{
        CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
    },
};
use shared::SharedFrame;

use crate::camera::Camera;

pub struct CameraNokhwa {
    hw: NokhwaCamera,
}

impl CameraNokhwa {
    /// Creates a new access point for a camera located at the provided index.
    pub fn new(index: u32, width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let camera_index = CameraIndex::Index(index);

        let resolution = Resolution::new(width, height);
        let camera_format = CameraFormat::new(resolution, FrameFormat::YUYV, 30);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(camera_format));

        let mut hw = NokhwaCamera::new(camera_index, requested)?;

        hw.open_stream()?;

        Ok(Self { hw })
    }
}

impl Camera for CameraNokhwa {
    /// Captures and returns a frame from the camera.
    fn grab_frame(&mut self) -> Result<SharedFrame, Box<dyn Error>> {
        let frame_buffer = self.hw.frame()?;

        let rgb_image = frame_buffer.decode_image::<RgbFormat>()?;

        let width = rgb_image.width();
        let height = rgb_image.height();

        let frame_data = rgb_image.into_raw();

        let shared = SharedFrame {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            width,
            height,
            data: Arc::new(frame_data),
        };

        Ok(shared)
    }
}
