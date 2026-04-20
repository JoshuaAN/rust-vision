import { useState, useEffect, useRef } from 'react';
import JMuxer from 'jmuxer';
import './index.css';

// ---------------------------------------------------------
// TypeScript Interfaces
// ---------------------------------------------------------
interface SystemMetrics {
  fps: number;
  latency: number;
}

interface Detection {
  id: number;
  label: string;
  conf: number;
  x?: number;
  y?: number;
}

interface AprilTag {
  id: number;
  distance: number;
  angle: number;
}

export default function App() {
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [metrics, setMetrics] = useState<SystemMetrics>({ fps: 30, latency: 12.4 });
  
  // Apply our custom interfaces to the state arrays
  const [_detections, setDetections] = useState<Detection[]>([]);
  const [_tags, setTags] = useState<AprilTag[]>([]);
  
  const videoRef = useRef<HTMLVideoElement>(null);

  // ---------------------------------------------------------
  // WebSocket & Video Player Setup
  // ---------------------------------------------------------
  useEffect(() => {
    if (!videoRef.current) return;

    const jmuxer = new JMuxer({
      node: videoRef.current,
      mode: 'video',
      flushingTime: 0,
      fps: 30,
      debug: false,
    });

    const ws = new WebSocket('ws://localhost:5800/ws');
    ws.binaryType = 'arraybuffer';

    ws.onopen = () => {
      console.log('Connected to video stream');
      setIsConnected(true);
    };

    ws.onmessage = (event) => {
      jmuxer.feed({ video: new Uint8Array(event.data) });
    };

    ws.onclose = () => {
      console.log('Disconnected from video stream');
      setIsConnected(false);
    };

    return () => {
      ws.close();
      jmuxer.destroy();
    };
  }, []);

  // ---------------------------------------------------------
  // Mock Data & Telemetry (Clears the unused variable errors)
  // ---------------------------------------------------------
  useEffect(() => {
    // We will replace this with real WebSocket telemetry data later
    setDetections([
      { id: 1, label: 'sports ball', conf: 0.92 },
      { id: 2, label: 'person', conf: 0.85 },
    ]);

    setTags([
      { id: 4, distance: 1.2, angle: 15.5 },
      { id: 7, distance: 2.8, angle: -5.0 },
    ]);

    const interval = setInterval(() => {
      setMetrics({
        fps: Math.floor(Math.random() * 5 + 28), 
        latency: +(Math.random() * 5 + 10).toFixed(1)
      });
    }, 1000);
    
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex flex-col h-screen w-screen bg-gray-100 text-gray-900 font-sans">
      
      {/* --- TOP STATUS BAR --- */}
      <header className="h-14 bg-white border-b border-gray-300 flex items-center justify-between px-6 shrink-0 shadow-sm z-10">
        <div className="flex items-center gap-3">
          <div className={`w-3 h-3 rounded-full ${isConnected ? 'bg-gray-800' : 'bg-gray-300 border border-gray-400'}`} />
          <h1 className="font-bold text-lg tracking-wider text-gray-800">VISION ORCHESTRATOR</h1>
        </div>
        
        <div className="flex gap-6 text-sm font-mono text-gray-500">
          <div className="flex flex-col items-end">
            <span className="text-[10px] text-gray-400 uppercase tracking-widest font-sans font-semibold">Camera FPS</span>
            <span className="text-gray-900 font-bold text-base leading-none">{metrics.fps}</span>
          </div>
          <div className="flex flex-col items-end">
            <span className="text-[10px] text-gray-400 uppercase tracking-widest font-sans font-semibold">AI Latency</span>
            <span className="text-gray-900 font-bold text-base leading-none">{metrics.latency}ms</span>
          </div>
        </div>
      </header>

      {/* --- MAIN WORKSPACE --- */}
      <div className="flex flex-1 overflow-hidden">
        
        {/* CENTER VIDEO FEED */}
        <main className="flex-1 flex flex-col items-center justify-center p-6 bg-gray-100 relative">
          <div className="w-full max-w-5xl aspect-video bg-gray-200 rounded-lg border-2 border-gray-300 overflow-hidden shadow-lg relative flex items-center justify-center">
            
            <video 
              ref={videoRef}
              autoPlay 
              muted 
              className="w-full h-full object-contain bg-black"
            />
            
            <div className="absolute inset-0 flex items-center justify-center pointer-events-none opacity-40">
              <div className="w-full h-[1px] bg-gray-900 absolute" />
              <div className="h-full w-[1px] bg-gray-900 absolute" />
              <div className="w-8 h-8 border-2 border-gray-900 rounded-full absolute" />
            </div>

            {!isConnected && (
              <div className="absolute inset-0 bg-gray-200/80 backdrop-blur-sm flex flex-col items-center justify-center">
                <div className="w-8 h-8 border-4 border-gray-400 border-t-gray-800 rounded-full animate-spin mb-4" />
                <p className="text-gray-800 font-bold tracking-widest animate-pulse">AWAITING STREAM...</p>
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}