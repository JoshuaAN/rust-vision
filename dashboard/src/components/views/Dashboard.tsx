import { useState } from 'react';
import { 
  Box, Typography, Paper, Slider, Stack, Divider, 
  Select, MenuItem, Tabs, Tab, Chip 
} from '@mui/material';
import Grid from '@mui/material/Grid';
import { WebRtcPlayer } from '../stream/WebRtcPlayer';

export const DashboardView = () => {
  const [pipeline, setPipeline] = useState(0);
  const [tabValue, setTabValue] = useState('1');

  // Add state for the sliders so we can display their current values
  const [exposure, setExposure] = useState<number>(20);
  const [decimation, setDecimation] = useState<number>(2);
  const [confidence, setConfidence] = useState<number>(0.6);

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', p: 2, gap: 2 }}>
      
      <Grid container spacing={3} sx={{ flexGrow: 1, height: '100%' }}>
        
        {/* LEFT COLUMN: Configuration */}
        <Grid size={{ xs: 12, md: 5 }} sx={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 3 }}>
          
          {/* TOP: Input Processing (Universal) */}
          <Paper sx={{ p: 3, bgcolor: 'background.paper', borderRadius: 2 }}>
            <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main' }}>Camera Settings</Typography>
            <Divider sx={{ my: 1.5 }} />
            <Stack spacing={2.5}>
              <Box>
                <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 0.5 }}>PIPELINE</Typography>
                <Select 
                  fullWidth size="small" 
                  value={pipeline} 
                  onChange={(e) => setPipeline(Number(e.target.value))}
                >
                  <MenuItem value={0}>AprilTags 36h11</MenuItem>
                  <MenuItem value={1}>Neural Detector</MenuItem>
                </Select>
              </Box>
              <Box>
                <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 0.5 }}>CAPTURE RESOLUTION</Typography>
                <Select fullWidth size="small" defaultValue="1280x720">
                  <MenuItem value="640x480">640 x 480 (High FPS)</MenuItem>
                  <MenuItem value="800x600">800 x 600</MenuItem>
                  <MenuItem value="1280x720">1280 x 720 (Standard)</MenuItem>
                </Select>
              </Box>
              
              {/* EXPOSURE SLIDER WITH VALUE READOUT */}
              <Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary' }}>EXPOSURE</Typography>
                  <Typography variant="caption" sx={{ fontWeight: 800, color: 'primary.main', fontFamily: 'monospace' }}>{exposure}</Typography>
                </Box>
                <Slider 
                  value={exposure} 
                  onChange={(_, val) => setExposure(val as number)}
                  size="small" 
                />
              </Box>

            </Stack>
          </Paper>

          {/* BOTTOM: Specialized Tuning Tabs */}
          <Paper sx={{ flexGrow: 1, borderRadius: 2, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
            <Box sx={{ borderBottom: 1, borderColor: 'divider', bgcolor: 'rgba(255,255,255,0.02)' }}>
              <Tabs value={tabValue} onChange={(_, v) => setTabValue(v)} variant="fullWidth">
                <Tab label="Input" value="1" sx={{ fontSize: '0.7rem', fontWeight: 700 }} />
                <Tab label="Detector" value="2" sx={{ fontSize: '0.7rem', fontWeight: 700 }} />
                <Tab label="Output" value="3" sx={{ fontSize: '0.7rem', fontWeight: 700 }} />
              </Tabs>
            </Box>

            <Box sx={{ p: 3, flexGrow: 1 }}>
              {tabValue === '1' && (
                <Stack spacing={3}>
                  <Typography variant="overline" sx={{ color: 'primary.main', fontWeight: 800 }}>
                    {pipeline === 0 ? 'AprilTag Config' : 'Neural Config'}
                  </Typography>
                  {pipeline === 0 ? (
                    <>
                      {/* DECIMATION SLIDER WITH VALUE READOUT */}
                      <Box>
                        <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
                          <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary' }}>DECIMATION</Typography>
                          <Typography variant="caption" sx={{ fontWeight: 800, color: 'primary.main', fontFamily: 'monospace' }}>{decimation}</Typography>
                        </Box>
                        <Slider 
                          value={decimation} 
                          onChange={(_, val) => setDecimation(val as number)}
                          min={1} max={4} step={1} marks 
                          size="small"
                        />
                      </Box>
                      <Box>
                        <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 0.5 }}>THREADS</Typography>
                        <Select fullWidth size="small" defaultValue={4}>
                          <MenuItem value={1}>1 Thread</MenuItem>
                          <MenuItem value={4}>4 Threads</MenuItem>
                        </Select>
                      </Box>
                    </>
                  ) : (
                    <Box>
                      {/* CONFIDENCE SLIDER WITH VALUE READOUT */}
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
                        <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary' }}>CONFIDENCE THRESHOLD</Typography>
                        <Typography variant="caption" sx={{ fontWeight: 800, color: 'primary.main', fontFamily: 'monospace' }}>{confidence.toFixed(2)}</Typography>
                      </Box>
                      <Slider 
                        value={confidence} 
                        onChange={(_, val) => setConfidence(val as number)}
                        min={0} max={1} step={0.01} 
                        size="small"
                      />
                    </Box>
                  )}
                </Stack>
              )}
              {tabValue === '3' && (
                <Stack spacing={3}>
                  <Typography variant="overline" sx={{ color: 'primary.main', fontWeight: 800 }}>
                    Stream Settings
                  </Typography>
                  
                  <Box>
                    <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 0.5 }}>STREAM RESOLUTION</Typography>
                    <Select fullWidth size="small" defaultValue="640x480">
                      <MenuItem value="320x240">320 x 240 (Low Bandwidth)</MenuItem>
                      <MenuItem value="640x480">640 x 480 (Standard)</MenuItem>
                      <MenuItem value="1280x720">1280 x 720 (High Quality)</MenuItem>
                    </Select>
                  </Box>
                </Stack>
              )}
            </Box>
          </Paper>
        </Grid>

        {/* RIGHT COLUMN: Output */}
        <Grid size={{ xs: 12, md: 7 }} sx={{ display: 'flex', flexDirection: 'column', height: '100%', gap: 2 }}>
          
          {/* Video Viewport */}
          <Paper sx={{ bgcolor: '#000', borderRadius: 2, overflow: 'hidden', position: 'relative' }}>
            <WebRtcPlayer />
            
            <Box sx={{ 
              position: 'absolute', top: 12, right: 12, 
              display: 'flex', gap: 1 
            }}>
              <Chip label="50 FPS" size="small" sx={{ bgcolor: 'rgba(0,0,0,0.6)', color: 'success.main', fontWeight: 900, borderRadius: 1 }} />
              <Chip label="1280x720" size="small" sx={{ bgcolor: 'rgba(0,0,0,0.6)', color: 'text.secondary', borderRadius: 1 }} />
            </Box>
          </Paper>

          {/* Target List */}
          <Paper sx={{ p: 2, flexGrow: 1, bgcolor: 'background.paper', borderRadius: 2 }}>
            <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', px: 1 }}>Target Data</Typography>
            <Divider sx={{ my: 1 }} />
            {/* Target Rows Go Here */}
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
};