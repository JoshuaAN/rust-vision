use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, watch};
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264};
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;
use webrtc::media::Sample;
use bytes::Bytes;

/// Manages active WebRTC connections and broadcasts the H.264 feed to them.
#[derive(Clone)]
pub struct WebRtcBroadcaster {
    api: Arc<webrtc::api::API>,
    // Thread-safe list of active video tracks (one for each connected browser)
    active_tracks: Arc<Mutex<Vec<Arc<TrackLocalStaticSample>>>>,
}

impl WebRtcBroadcaster {
    /// Initializes the WebRTC engine and starts the background broadcasting loop.
    pub fn start(mut h264_rx: watch::Receiver<Vec<u8>>) -> Self {
        // 1. Setup the WebRTC Media Engine to strictly handle H.264
        let mut m = MediaEngine::default();
        m.register_default_codecs().expect("Failed to register codecs");

        let api = Arc::new(APIBuilder::new().with_media_engine(m).build());
        let active_tracks: Arc<Mutex<Vec<Arc<TrackLocalStaticSample>>>> = Arc::new(Mutex::new(Vec::new()));

        // 2. Spawn the Broadcasting Loop
        let tracks_clone = active_tracks.clone();
        tokio::spawn(async move {
            let mut last_timestamp = SystemTime::now();

            loop {
                if h264_rx.changed().await.is_err() {
                    println!("Broadcaster: Video feed closed.");
                    break;
                }

                let frame_data = h264_rx.borrow().clone();
                if frame_data.is_empty() {
                    continue;
                }

                // Calculate duration since the last frame for WebRTC sync
                let now = SystemTime::now();
                let duration = now.duration_since(last_timestamp).unwrap_or(Duration::from_millis(33));
                last_timestamp = now;

                let sample = Sample {
                    data: Bytes::from(frame_data),
                    duration,
                    ..Default::default()
                };

                // Blast the sample to all connected clients
                let mut tracks = tracks_clone.lock().await;
                let mut i = 0;
                while i < tracks.len() {
                    if let Err(e) = tracks[i].write_sample(&sample).await {
                        // If writing fails (e.g., client disconnected), remove the track
                        println!("Client disconnected, removing track. Error: {}", e);
                        tracks.remove(i);
                    } else {
                        i += 1;
                    }
                }
            }
        });

        Self { api, active_tracks }
    }

    /// Takes an SDP Offer from the Dashboard crate, creates a PeerConnection, 
    /// attaches a video track, and returns the SDP Answer.
    pub async fn handle_sdp_offer(&self, offer_sdp: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 1. Create a new Peer Connection
        let config = RTCConfiguration {
            ice_servers: vec![], // For local networks, no STUN/TURN needed yet
            ..Default::default()
        };
        let peer_connection = Arc::new(self.api.new_peer_connection(config).await?);

        // 2. Create a new H.264 Video Track for this specific user
        let video_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: MIME_TYPE_H264.to_owned(),
                ..Default::default()
            },
            "video".to_owned(),
            "webrtc-rs".to_owned(),
        ));

        // 3. Add the track to the peer connection
        peer_connection.add_track(Arc::clone(&video_track) as Arc<dyn TrackLocal + Send + Sync>).await?;

        // 4. Add the track to our broadcaster's active list
        self.active_tracks.lock().await.push(video_track);

        // 5. Cleanup when the connection drops
        peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            if s == RTCPeerConnectionState::Failed || s == RTCPeerConnectionState::Closed {
                println!("WebRTC Peer Connection Closed");
            }
            Box::pin(async {})
        }));

        // 6. Perform the SDP Handshake
        let offer = RTCSessionDescription::offer(offer_sdp.to_owned())?;
        peer_connection.set_remote_description(offer).await?;

        let answer = peer_connection.create_answer(None).await?;
        peer_connection.set_local_description(answer.clone()).await?;

        Ok(answer.sdp)
    }
}