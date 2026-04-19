use camera::CameraNode;
use streamer::StreamerNode;
use std::future;

#[tokio::main]
async fn main() {
    // 1. Start the camera thread
    let cam_node = CameraNode::start(0, 640, 480);

    // 2. Start the streamer daemon (This blocks forever to keep the app running)
    StreamerNode::start(cam_node.frame_rx, 640, 480, 5800).await;
}