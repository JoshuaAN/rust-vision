use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct SharedFrame {
    pub timestamp_ms: u64,
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<u8>>,
}

#[derive(Clone, Default)]
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
}