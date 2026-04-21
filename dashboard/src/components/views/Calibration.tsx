// src/components/views/CalibrationView.tsx
import React, { useState } from 'react';
import { 
  Box, Typography, Paper, Divider, Stack, 
  Select, MenuItem, TextField, Button, Chip, 
  LinearProgress, Alert
} from '@mui/material';
import Grid from '@mui/material/Grid';
import PhotoCameraIcon from '@mui/icons-material/PhotoCamera';
import CalculateIcon from '@mui/icons-material/Calculate';
import SaveIcon from '@mui/icons-material/Save';
import { WebRtcPlayer } from '../stream/WebRtcPlayer'; // Assuming you have this from the Dashboard

export const CalibrationView = () => {
  // Setup State
  const [targetType, setTargetType] = useState('charuco');
  const [capturedFrames, setCapturedFrames] = useState(0);
  const [isCalibrated, setIsCalibrated] = useState(false);
  const [isComputing, setIsComputing] = useState(false);

  // Mock functions for the UI
  const handleCapture = () => setCapturedFrames(prev => prev + 1);
  const handleClear = () => { setCapturedFrames(0); setIsCalibrated(false); };
  
  const handleCompute = () => {
    setIsComputing(true);
    // Simulate computing delay
    setTimeout(() => {
      setIsComputing(false);
      setIsCalibrated(true);
    }, 2000);
  };

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', p: 2, gap: 2, overflowY: 'auto' }}>
      <Grid container spacing={3} sx={{ flexGrow: 1 }}>
        
        {/* LEFT COLUMN: Configuration & Actions */}
        <Grid size={{ xs: 12, md: 5 }} sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          
          {/* Target Configuration */}
          <Paper sx={{ p: 4, bgcolor: 'background.paper', borderRadius: 2 }}>
            <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', fontSize: '0.85rem' }}>
              Target Configuration
            </Typography>
            <Divider sx={{ mt: 1, mb: 3 }} />

            <Stack spacing={3}>
              <Box>
                <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                  TARGET TYPE
                </Typography>
                <Select 
                  fullWidth size="small" 
                  value={targetType} 
                  onChange={(e) => setTargetType(e.target.value)}
                >
                  <MenuItem value="charuco">ChArUco Board (Recommended)</MenuItem>
                  <MenuItem value="checkerboard">Standard Checkerboard</MenuItem>
                </Select>
              </Box>

              <Box sx={{ display: 'flex', gap: 2 }}>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    CORNERS X
                  </Typography>
                  <TextField 
                    fullWidth size="small" defaultValue="8"
                    slotProps={{ htmlInput: { inputMode: 'numeric', pattern: '[0-9]*' } }}
                  />
                </Box>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    CORNERS Y
                  </Typography>
                  <TextField 
                    fullWidth size="small" defaultValue="6"
                    slotProps={{ htmlInput: { inputMode: 'numeric', pattern: '[0-9]*' } }}
                  />
                </Box>
              </Box>

              <Box sx={{ display: 'flex', gap: 2 }}>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    SQUARE SIZE (mm)
                  </Typography>
                  <TextField 
                    fullWidth size="small" defaultValue="30"
                    slotProps={{ htmlInput: { inputMode: 'numeric' } }}
                  />
                </Box>
                <Box sx={{ flex: 1, opacity: targetType === 'charuco' ? 1 : 0.3, pointerEvents: targetType === 'charuco' ? 'auto' : 'none' }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    MARKER SIZE (mm)
                  </Typography>
                  <TextField 
                    fullWidth size="small" defaultValue="22"
                    slotProps={{ htmlInput: { inputMode: 'numeric' } }}
                  />
                </Box>
              </Box>
            </Stack>
          </Paper>

          {/* Workflow Actions */}
          <Paper sx={{ p: 4, bgcolor: 'background.paper', borderRadius: 2, flexGrow: 1, display: 'flex', flexDirection: 'column' }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 1 }}>
              <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', fontSize: '0.85rem' }}>
                Capture Workflow
              </Typography>
              <Chip 
                label={`${capturedFrames} Frames`} 
                size="small" 
                color={capturedFrames >= 15 ? 'success' : capturedFrames > 0 ? 'warning' : 'default'}
                sx={{ fontWeight: 800, borderRadius: 1 }}
              />
            </Box>
            <Divider sx={{ mb: 3 }} />

            <Stack spacing={2} sx={{ flexGrow: 1 }}>
              <Button 
                variant="contained" 
                size="large"
                startIcon={<PhotoCameraIcon />}
                onClick={handleCapture}
                sx={{ py: 2, fontWeight: 800, fontSize: '1rem' }}
              >
                Capture Frame
              </Button>

              {/* Progress bar visualizer for recommended frames (Target: 15+) */}
              <Box sx={{ px: 1 }}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 0.5 }}>
                  <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700 }}>Progress</Typography>
                  <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700 }}>15+ Recommended</Typography>
                </Box>
                <LinearProgress 
                  variant="determinate" 
                  value={Math.min((capturedFrames / 15) * 100, 100)} 
                  color={capturedFrames >= 15 ? 'success' : 'primary'}
                  sx={{ height: 6, borderRadius: 3 }}
                />
              </Box>

              <Box sx={{ display: 'flex', gap: 2, mt: 'auto !important' }}>
                <Button 
                  variant="outlined" 
                  color="error" 
                  onClick={handleClear}
                  disabled={capturedFrames === 0}
                  sx={{ flex: 1, fontWeight: 700 }}
                >
                  Clear All
                </Button>
                <Button 
                  variant="contained" 
                  color="secondary"
                  startIcon={<CalculateIcon />}
                  onClick={handleCompute}
                  disabled={capturedFrames < 5 || isComputing}
                  sx={{ flex: 2, fontWeight: 800 }}
                >
                  {isComputing ? 'Computing...' : 'Compute Calibration'}
                </Button>
              </Box>
            </Stack>
          </Paper>

        </Grid>

        {/* RIGHT COLUMN: Camera View & Results */}
        <Grid size={{ xs: 12, md: 7 }} sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
          
          {/* Video Feed */}
          <Paper sx={{ bgcolor: '#000', borderRadius: 2, overflow: 'hidden', border: '1px solid', borderColor: 'divider', position: 'relative' }}>
            <WebRtcPlayer />
            {/* Overlay instruction */}
            <Box sx={{ position: 'absolute', bottom: 16, left: '50%', transform: 'translateX(-50%)', bgcolor: 'rgba(0,0,0,0.7)', px: 3, py: 1, borderRadius: 8, backdropFilter: 'blur(4px)' }}>
              <Typography variant="caption" sx={{ color: '#fff', fontWeight: 700, letterSpacing: 0.5 }}>
                Hold the target steady in various angles and distances
              </Typography>
            </Box>
          </Paper>

          {/* Results Panel */}
          <Paper sx={{ p: 4, bgcolor: 'background.paper', borderRadius: 2, flexGrow: 1, opacity: isCalibrated ? 1 : 0.5, transition: 'opacity 0.3s' }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <Typography variant="overline" sx={{ fontWeight: 800, color: isCalibrated ? 'success.main' : 'text.disabled', fontSize: '0.85rem' }}>
                Calibration Results
              </Typography>
              {isCalibrated && (
                <Button variant="contained" color="success" size="small" startIcon={<SaveIcon />} sx={{ fontWeight: 800, borderRadius: 1.5 }}>
                  Save to Camera
                </Button>
              )}
            </Box>
            <Divider sx={{ mt: 1, mb: 3 }} />

            {!isCalibrated ? (
              <Box sx={{ display: 'flex', height: '100%', alignItems: 'center', justifyContent: 'center', pb: 4 }}>
                <Typography variant="body2" sx={{ color: 'text.secondary', fontWeight: 600 }}>
                  Capture frames and compute to view intrinsics.
                </Typography>
              </Box>
            ) : (
              <Stack spacing={3}>
                <Alert severity="success" sx={{ bgcolor: 'rgba(46, 125, 50, 0.1)', '& .MuiAlert-icon': { color: 'success.main' } }}>
                  Calibration successful! Reprojection Error (RMSE): <strong>0.24 px</strong>
                </Alert>
                
                <Box sx={{ display: 'flex', gap: 4 }}>
                  <Box sx={{ flex: 1 }}>
                    <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700, display: 'block', mb: 1 }}>CAMERA MATRIX (Intrinsics)</Typography>
                    <Box sx={{ p: 2, bgcolor: 'rgba(0,0,0,0.2)', borderRadius: 1, border: '1px solid', borderColor: 'divider' }}>
                      <Typography sx={{ fontFamily: 'monospace', fontSize: '0.9rem', whiteSpace: 'pre', color: 'text.primary' }}>
                        [ 1024.5,    0.0,  640.2 ]{'\n'}
                        [    0.0, 1024.5,  360.5 ]{'\n'}
                        [    0.0,    0.0,    1.0 ]
                      </Typography>
                    </Box>
                  </Box>

                  <Box sx={{ flex: 1 }}>
                    <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700, display: 'block', mb: 1 }}>DISTORTION COEFFICIENTS</Typography>
                    <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 1.5 }}>
                      {['k1: 0.1245', 'k2: -0.0452', 'p1: 0.0012', 'p2: -0.0004', 'k3: 0.0000'].map((coef, i) => (
                        <Box key={i} sx={{ p: 1, bgcolor: 'rgba(0,0,0,0.2)', borderRadius: 1, border: '1px solid', borderColor: 'divider' }}>
                          <Typography sx={{ fontFamily: 'monospace', fontSize: '0.85rem', color: 'text.secondary' }}>
                            {coef}
                          </Typography>
                        </Box>
                      ))}
                    </Box>
                  </Box>
                </Box>
              </Stack>
            )}
          </Paper>

        </Grid>
      </Grid>
    </Box>
  );
};