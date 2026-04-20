use std::error::Error;

use image::{DynamicImage, GenericImageView, RgbImage, imageops::FilterType};
use ndarray::{Array4, ArrayViewD, Axis};
use ort::session::Session;
use shared::{ObjectDetection, SharedFrame};

use crate::object_detector::ObjectDetector;

const COCO_LABELS: &[&str] = &[
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat",
    "traffic light", "fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat", "dog",
    "horse", "sheep", "cow", "elephant", "bear", "zebra", "giraffe", "backpack", "umbrella",
    "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard", "sports ball", "kite",
    "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket", "bottle",
    "wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange",
    "broccoli", "carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch", "potted plant",
    "bed", "dining table", "toilet", "tv", "laptop", "mouse", "remote", "keyboard", "cell phone",
    "microwave", "oven", "toaster", "sink", "refrigerator", "book", "clock", "vase", "scissors",
    "teddy bear", "hair drier", "toothbrush",
];

pub struct ObjectDetectorOnnx {
    session: Session,
}

impl ObjectDetectorOnnx {
    pub fn new(model_path: &str) -> Result<Self, Box<dyn Error>> {
        let session = Session::builder()?
            .with_intra_threads(4)?
            .commit_from_file(model_path)?;

        Ok(Self { session })
    }
}

impl ObjectDetector for ObjectDetectorOnnx {
    fn detect(&mut self, frame: &SharedFrame) -> Vec<ObjectDetection> {
        // 1. Reconstruct a DynamicImage from the SharedFrame's raw RGB bytes
        let rgb_image = RgbImage::from_raw(frame.width, frame.height, frame.data.to_vec())
            .expect("Failed to create RgbImage: frame data size doesn't match width * height * 3");
            
        let dyn_image = DynamicImage::ImageRgb8(rgb_image);
        let (img_width, img_height) = dyn_image.dimensions();

        // 2. Resize and Pre-process
        let resized = dyn_image.resize_exact(640, 640, FilterType::Triangle);
        let mut input = Array4::<f32>::zeros((1, 3, 640, 640));

        for (x, y, rgb) in resized.pixels() {
            input[[0, 0, y as usize, x as usize]] = rgb[0] as f32 / 255.0;
            input[[0, 1, y as usize, x as usize]] = rgb[1] as f32 / 255.0;
            input[[0, 2, y as usize, x as usize]] = rgb[2] as f32 / 255.0;
        }

        let (shape_usize, data) = {
            let input_tensor = ort::value::Tensor::from_array(input).unwrap();
            let outputs = self.session.run(ort::inputs![input_tensor]).unwrap();

            let output_tensor = outputs.get("output0").expect("Output 'output0' not found");
            let (shape_i64, data_slice) = output_tensor.try_extract_tensor::<f32>().unwrap();

            let shape: Vec<usize> = shape_i64.iter().map(|&x| x as usize).collect();

            // Copy data so we don't hold references to the session
            (shape, data_slice.to_vec())
        }; 
        
        let output_view = ndarray::ArrayViewD::from_shape(ndarray::IxDyn(&shape_usize), &data).unwrap();

        self.parse_yolo_output(output_view, img_width as f32, img_height as f32)
    }
}

impl ObjectDetectorOnnx {
    fn parse_yolo_output(
        &self,
        output: ArrayViewD<f32>,
        img_w: f32,
        img_h: f32,
    ) -> Vec<ObjectDetection> {
        let mut detections = Vec::new();
        let conf_threshold = 0.5;

        // YOLOv8 output is [1, 84, 8400]. We remove the batch dimension.
        let view = output.view().remove_axis(Axis(0));

        for i in 0..8400 {
            let mut max_conf = 0.0;
            let mut class_id = 0;

            for c in 4..84 {
                let conf = view[[c, i]];
                if conf > max_conf {
                    max_conf = conf;
                    class_id = c - 4;
                }
            }

            if max_conf > conf_threshold {
                let cx = view[[0, i]];
                let cy = view[[1, i]];
                let w = view[[2, i]];
                let h = view[[3, i]];

                let x_min = (cx - w / 2.0) * (img_w / 640.0);
                let y_min = (cy - h / 2.0) * (img_h / 640.0);
                let x_max = (cx + w / 2.0) * (img_w / 640.0);
                let y_max = (cy + h / 2.0) * (img_h / 640.0);

                let label_name = COCO_LABELS
                    .get(class_id as usize)
                    .unwrap_or(&"unknown")
                    .to_string();

                detections.push(ObjectDetection {
                    label: label_name,
                    confidence: max_conf,
                    box_2d: [x_min, y_min, x_max, y_max],
                });
            }
        }

        self.apply_nms(detections, 0.45)
    }

    fn apply_nms(
        &self,
        mut detections: Vec<ObjectDetection>,
        iou_threshold: f32,
    ) -> Vec<ObjectDetection> {
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let mut kept = Vec::new();
        while !detections.is_empty() {
            let best = detections.remove(0);

            detections
                .retain(|item| intersection_over_union(&best.box_2d, &item.box_2d) < iou_threshold);

            kept.push(best);
        }
        kept
    }
}

fn intersection_over_union(box_a: &[f32; 4], box_b: &[f32; 4]) -> f32 {
    let x_overlap = 0.0f32.max(box_a[2].min(box_b[2]) - box_a[0].max(box_b[0]));
    let y_overlap = 0.0f32.max(box_a[3].min(box_b[3]) - box_a[1].max(box_b[1]));
    let intersection = x_overlap * y_overlap;

    let area_a = (box_a[2] - box_a[0]) * (box_a[3] - box_a[1]);
    let area_b = (box_b[2] - box_b[0]) * (box_b[3] - box_b[1]);
    let union = area_a + area_b - intersection;

    intersection / union
}