import { useState, useEffect, useRef } from 'react';
import './index.css';

// --- Types ---
interface AprilTag {
  id: number;
  corners: [number, number][]; 
}

interface ObjectDetection {
  label: string;
  confidence: number;
  box_2d: [number, number, number, number]; 
}

interface PipelineResult {
  frame_timestamp: number;
  tags: AprilTag[];
  objects: ObjectDetection[];
}

export default function App() {
  const [isConnected, setIsConnected] = useState(false);
  const [showOverlays, setShowOverlays] = useState(true);
  
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const metadataBuffer = useRef<Map<number, PipelineResult>>(new Map());

  // ---------------------------------------------------------
  // 1. WEBSOCKET: Receive & Buffer Normalized Metadata
  // ---------------------------------------------------------
  useEffect(() => {
    const wsUrl = `ws://${window.location.hostname}:8080/ws`;
    const ws = new WebSocket(wsUrl);
    
    ws.onmessage = (event) => {
      try {
        const data: PipelineResult = JSON.parse(event.data);
        metadataBuffer.current.set(data.frame_timestamp, data);

        // Keep buffer lean (last 50 frames)
        if (metadataBuffer.current.size > 50) {
          const oldestKey = Math.min(...metadataBuffer.current.keys());
          metadataBuffer.current.delete(oldestKey);
        }
      } catch (err) {
        console.error("Failed to parse WebSocket JSON:", err);
      }
    };

    return () => ws.close();
  }, []);

  // ---------------------------------------------------------
  // 2. RENDER LOOP: 50ms Sync & Scaling
  // ---------------------------------------------------------
  useEffect(() => {
    if (!videoRef.current || !canvasRef.current) return;

    const drawFrame = (_now: number, metadata: any) => {
      const canvas = canvasRef.current;
      const ctx = canvas?.getContext('2d');
      if (!ctx || !canvas) return;

      // Clear previous frame
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      if (showOverlays) {
        // Sync: Use Absolute Capture Time or guess based on local latency
        const targetTs = metadata.captureTime || (Date.now() - 60); 
        
        let closestMatch: PipelineResult | null = null;
        let minDiff = Infinity;

        // Pass 1: Strict 50ms sync window
        metadataBuffer.current.forEach((val, ts) => {
          const diff = Math.abs(ts - targetTs);
          if (diff < 50 && diff < minDiff) { 
            minDiff = diff;
            closestMatch = val;
          }
        });

        // Pass 2: Fallback to newest packet to prevent visual stutter
        if (!closestMatch && metadataBuffer.current.size > 0) {
          const newestTs = Math.max(...metadataBuffer.current.keys());
          closestMatch = metadataBuffer.current.get(newestTs) || null;
        }

        if (closestMatch) {
          renderOverlay(ctx, closestMatch, canvas.width, canvas.height);
        }
      }

      // Schedule next frame
      videoRef.current?.requestVideoFrameCallback(drawFrame);
    };

    const handle = videoRef.current.requestVideoFrameCallback(drawFrame);
    return () => videoRef.current?.cancelVideoFrameCallback(handle);
  }, [showOverlays]);

  // ---------------------------------------------------------
  // 3. DRAWING LOGIC: Math for Normalized Coordinates
  // ---------------------------------------------------------
  const renderOverlay = (ctx: CanvasRenderingContext2D, data: PipelineResult, width: number, height: number) => {
    ctx.lineWidth = 3;

    // Draw YOLO Objects
    data.objects?.forEach(obj => {
      if (!obj.box_2d || obj.box_2d.length < 4) return;

      const [x1, y1, x2, y2] = obj.box_2d;

      // SCALE: Multiply 0.0-1.0 coords by canvas dimensions
      const startX = x1 * width;
      const startY = y1 * height;
      const boxW = (x2 - x1) * width;
      const boxH = (y2 - y1) * height;

      // Draw Box
      ctx.strokeStyle = '#00f2ff';
      ctx.strokeRect(startX, startY, boxW, boxH);
      
      // Draw Label Background
      ctx.fillStyle = 'rgba(0, 242, 255, 0.2)';
      ctx.fillRect(startX, startY - 22, boxW, 22);

      // Draw Text
      ctx.fillStyle = '#00f2ff';
      ctx.font = 'bold 14px Inter, sans-serif';
      ctx.fillText(`${obj.label} ${(obj.confidence * 100).toFixed(0)}%`, startX + 4, startY - 6);
    });

    // Draw AprilTags
    data.tags?.forEach(tag => {
      if (!tag.corners || tag.corners.length < 4) return;

      ctx.strokeStyle = '#39ff14';
      ctx.beginPath();
      
      // Scale all 4 corners
      ctx.moveTo(tag.corners[0][0] * width, tag.corners[0][1] * height);
      tag.corners.forEach(([cx, cy]) => ctx.lineTo(cx * width, cy * height));
      
      ctx.closePath();
      ctx.stroke();
      
      // Draw ID Text
      ctx.fillStyle = '#39ff14';
      ctx.font = 'bold 16px Inter, sans-serif';
      ctx.fillText(`ID: ${tag.id}`, tag.corners[0][0] * width, tag.corners[0][1] * height - 8);
    });
  };

  // ---------------------------------------------------------
  // 4. WEBRTC: Setup & Canvas Sync
  // ---------------------------------------------------------
  useEffect(() => {
    let pc: RTCPeerConnection;
    
    const startWebRTC = async () => {
      pc = new RTCPeerConnection({ iceServers: [] }); // Local network only
      pc.addTransceiver('video', { direction: 'recvonly' });
      
      pc.ontrack = (e) => {
        if (videoRef.current && e.streams[0]) {
          videoRef.current.srcObject = e.streams[0];
          
          // CRITICAL: Sync canvas resolution to actual stream resolution
          videoRef.current.onloadedmetadata = () => {
            if (videoRef.current && canvasRef.current) {
              const w = videoRef.current.videoWidth;
              const h = videoRef.current.videoHeight;
              canvasRef.current.width = w;
              canvasRef.current.height = h;
              setIsConnected(true);
              console.log(`Stream connected natively at ${w}x${h}`);
            }
          };
        }
      };
      
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      
      await new Promise<void>((r) => {
        if (pc.iceGatheringState === 'complete') r();
        else pc.onicecandidate = (e) => e.candidate === null && r();
      });

      const res = await fetch(`http://${window.location.hostname}:8080/webrtc/sdp`, {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: pc.localDescription?.sdp,
      });

      const answer = await res.text();
      await pc.setRemoteDescription({ type: 'answer', sdp: answer });
    };

    startWebRTC();
    return () => pc?.close();
  }, []);

  // ---------------------------------------------------------
  // UI RENDER
  // ---------------------------------------------------------
  return (
    <div className="flex flex-col h-screen w-screen bg-gray-950 text-gray-100 overflow-hidden font-sans">
      <header className="h-14 bg-gray-900 border-b border-gray-800 flex items-center justify-between px-6 shrink-0 shadow-sm z-10">
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500 animate-pulse shadow-[0_0_10px_rgba(239,68,68,0.7)]'}`} />
          <h1 className="font-bold text-lg tracking-wider text-white">LEMONLANTERN</h1>
        </div>
        <label className="flex items-center gap-2 text-sm font-semibold cursor-pointer bg-gray-800 hover:bg-gray-700 transition-colors px-4 py-1.5 rounded-md border border-gray-700">
          <input type="checkbox" className="accent-cyan-400" checked={showOverlays} onChange={e => setShowOverlays(e.target.checked)} />
          Vision Overlays
        </label>
      </header>

      <main className="flex-1 flex items-center justify-center p-6 bg-gray-950 relative">
        {/* SHRINK-WRAP CONTAINER: Forces canvas to perfectly align over video pixels */}
        <div className="relative inline-block border border-gray-800 rounded-lg overflow-hidden shadow-2xl bg-black">
          
          <video 
            ref={videoRef} 
            autoPlay 
            playsInline 
            muted 
            // Max bounds prevent it from pushing off-screen on smaller monitors
            className="block max-w-full max-h-[85vh] object-contain" 
          />
          
          <canvas 
            ref={canvasRef} 
            // Inset-0 pins the corners of the canvas to the corners of the shrink-wrapped video container
            className="absolute inset-0 pointer-events-none w-full h-full" 
          />

          {!isConnected && (
            <div className="absolute inset-0 bg-black/80 backdrop-blur-sm flex flex-col items-center justify-center">
              <div className="w-10 h-10 border-4 border-gray-600 border-t-cyan-400 rounded-full animate-spin mb-4" />
              <p className="text-cyan-400 font-mono tracking-widest uppercase text-sm">Waiting for WebRTC...</p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}