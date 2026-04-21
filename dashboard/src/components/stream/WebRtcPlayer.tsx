import React, { useState, useEffect, useRef } from 'react';
import { Box, Typography, CircularProgress } from '@mui/material';

export const WebRtcPlayer: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    let pc: RTCPeerConnection;

    const startWebRTC = async () => {
      // 1. Initialize Peer Connection (Local network)
      pc = new RTCPeerConnection({ iceServers: [] });
      
      // 2. Setup Transceiver to receive only
      pc.addTransceiver('video', { direction: 'recvonly' });

      // 3. Handle incoming stream
      pc.ontrack = (e) => {
        if (videoRef.current && e.streams[0]) {
          videoRef.current.srcObject = e.streams[0];
          
          videoRef.current.onloadedmetadata = () => {
            setIsConnected(true);
          };
        }
      };

      try {
        // 4. Create and set Local Offer
        const offer = await pc.createOffer();
        await pc.setLocalDescription(offer);

        // 5. Wait for ICE gathering to complete
        await new Promise<void>((resolve) => {
          if (pc.iceGatheringState === 'complete') {
            resolve();
          } else {
            pc.onicecandidate = (e) => {
              if (e.candidate === null) resolve();
            };
          }
        });

        // 6. Exchange SDP with Backend
        const res = await fetch(`http://${window.location.hostname}:8080/webrtc/sdp`, {
          method: 'POST',
          headers: { 'Content-Type': 'text/plain' },
          body: pc.localDescription?.sdp,
        });

        if (!res.ok) throw new Error('SDP Exchange Failed');

        const answer = await res.text();
        await pc.setRemoteDescription({ type: 'answer', sdp: answer });
      } catch (err) {
        console.error("WebRTC Connection Error:", err);
      }
    };

    startWebRTC();

    // Cleanup connection on unmount
    return () => {
      if (pc) pc.close();
    };
  }, []);

  return (
    <Box 
      sx={{ 
        position: 'relative', 
        width: '100%', 
        aspectRatio: '16/9', 
        bgcolor: '#000', 
        borderRadius: 2, 
        overflow: 'hidden', 
        border: '1px solid', 
        borderColor: 'divider',
        boxShadow: '0 4px 20px rgba(0,0,0,0.5)'
      }}
    >
      {/* Pure Video Stream */}
      <video
        ref={videoRef}
        autoPlay
        playsInline
        muted
        style={{
          width: '100%',
          height: '100%',
          objectFit: 'contain',
        }}
      />

      {/* Connection State Overlay */}
      {!isConnected && (
        <Box 
          sx={{ 
            position: 'absolute', 
            inset: 0, 
            display: 'flex', 
            flexDirection: 'column', 
            alignItems: 'center', 
            justifyContent: 'center', 
            bgcolor: 'rgba(23, 23, 23, 0.9)', // Matching your Carbon background
            gap: 2,
            zIndex: 10
          }}
        >
          <CircularProgress color="primary" size={32} thickness={5} />
          <Typography 
            color="text.secondary" 
            variant="body2" 
            sx={{ fontWeight: 700, letterSpacing: '0.05em' }}
          >
            INITIALIZING WEBRTC FEED...
          </Typography>
        </Box>
      )}
    </Box>
  );
};