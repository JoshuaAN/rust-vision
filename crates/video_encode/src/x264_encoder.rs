use std::error::Error;

use vision_core::{EncodedFrame, SharedFrame};
use x264::{Colorspace, Image, Plane, Preset, Setup, Tune};

use crate::encoder::VideoEncoder;

pub struct X264Encoder {
    encoder: x264::Encoder,
    width: u32,
    height: u32,
    pts: i64,
}

unsafe impl Send for X264Encoder {}

impl X264Encoder {
    pub fn new(width: u32, height: u32) -> Result<Self, Box<dyn Error>> {
        let encoder = Setup::preset(Preset::Ultrafast, Tune::None, false, true)
            .fps(30, 1)
            .build(Colorspace::I420, width as i32, height as i32)
            .map_err(|e| format!("Failed to build x264 encoder: {:?}", e))?;

        Ok(Self {
            encoder,
            width,
            height,
            pts: 0,
        })
    }
}

impl VideoEncoder for X264Encoder {
    fn encode(&mut self, frame: &SharedFrame) -> Result<EncodedFrame, Box<dyn Error>> {
        let (y, u, v) = rgb_to_i420(self.width as usize, self.height as usize, &frame.data);

        let plane_y = Plane { stride: self.width as i32, data: &y };
        let plane_u = Plane { stride: (self.width / 2) as i32, data: &u };
        let plane_v = Plane { stride: (self.width / 2) as i32, data: &v };

        let x264_image = Image::new(
            Colorspace::I420,
            self.width as i32,
            self.height as i32,
            &[plane_y, plane_u, plane_v],
        );

        // 2. Encode
        match self.encoder.encode(self.pts, x264_image) {
            Ok((data, _picture)) => {
                self.pts += 1;
                Ok(EncodedFrame { data: data.entirety().to_vec(), timestamp_ms: frame.timestamp_ms })
            }
            Err(_) => Err("Failed to encode frame".into()),
        }
    }
}

fn rgb_to_i420(width: usize, height: usize, rgb: &[u8]) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut y_plane = vec![0u8; width * height];
    let mut u_plane = vec![0u8; (width / 2) * (height / 2)];
    let mut v_plane = vec![0u8; (width / 2) * (height / 2)];

    for j in 0..height {
        for i in 0..width {
            let idx = (j * width + i) * 3;
            let r = rgb[idx] as f32;
            let g = rgb[idx + 1] as f32;
            let b = rgb[idx + 2] as f32;

            let y = 0.299 * r + 0.587 * g + 0.114 * b;
            y_plane[j * width + i] = y.clamp(0.0, 255.0) as u8;

            if j % 2 == 0 && i % 2 == 0 {
                let u = -0.1687 * r - 0.3313 * g + 0.5 * b + 128.0;
                let v = 0.5 * r - 0.4187 * g - 0.0813 * b + 128.0;
                let uv_idx = (j / 2) * (width / 2) + (i / 2);
                u_plane[uv_idx] = u.clamp(0.0, 255.0) as u8;
                v_plane[uv_idx] = v.clamp(0.0, 255.0) as u8;
            }
        }
    }
    (y_plane, u_plane, v_plane)
}