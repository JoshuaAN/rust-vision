use image::{DynamicImage, GenericImageView, imageops::FilterType};
use ndarray::{Array4, ArrayViewD, Axis, IxDyn};
use ort::{inputs, session::Session, value::Value};
use shared::ObjectDetection; // Ensure this matches your renamed crate

mod object_detector;
mod object_detector_onnx;

pub use object_detector::ObjectDetector;
pub use object_detector_onnx::ObjectDetectorOnnx;