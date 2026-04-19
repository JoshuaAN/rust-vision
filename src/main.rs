use camera::CameraNode; // Update this path if your export is different
use streamer::StreamerNode;
use minifb::{Key, Window, WindowOptions};

#[tokio::main]
async fn main() {
    let width = 640;
    let height = 480;
    
    println!("Booting FRC Vision Daemon...");

    // 1. Start the hardware capture thread
    let cam_node = CameraNode::start(0, width, height);
    let mut rx = cam_node.frame_rx.clone();

    // 2. Start the streamer daemon in the background
    // We spawn this so it doesn't block our main UI thread loop below!
    let streamer_rx = cam_node.frame_rx.clone();
    tokio::spawn(async move {
        StreamerNode::start(streamer_rx, width, height, 5800).await;
    });

    // 3. Setup the local debug window
    let mut window = Window::new(
        "Local Camera Raw Feed - Press ESC to exit",
        width as usize,
        height as usize,
        WindowOptions::default(),
    ).expect("Failed to open local window");

    // Limit the window to ~60 FPS to not thrash the UI thread unnecessarily
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = vec![0; (width * height) as usize];

    // 4. Main Event Loop (runs on the main thread for macOS compatibility)
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Wait for the newest frame to arrive from the camera thread
        if rx.changed().await.is_ok() {
            if let Some(frame) = rx.borrow().clone() {
                
                // minifb expects a 32-bit pixel format (0x00RRGGBB)
                // Our camera data is interleaved 8-bit RGB (R, G, B, R, G, B...)
                for i in 0..(width * height) as usize {
                    let r = frame.data[i * 3] as u32;
                    let g = frame.data[i * 3 + 1] as u32;
                    let b = frame.data[i * 3 + 2] as u32;
                    
                    // Bit-shift the colors into a single u32
                    buffer[i] = (r << 16) | (g << 8) | b;
                }

                // Push the buffer to the screen
                window
                    .update_with_buffer(&buffer, width as usize, height as usize)
                    .unwrap();
            }
        } else {
            // Camera channel closed, exit loop
            break;
        }
    }
    
    println!("Shutting down vision daemon.");
}