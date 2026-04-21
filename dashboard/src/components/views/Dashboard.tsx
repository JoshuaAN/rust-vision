// src/components/views/Dashboard.tsx
import { Box, Typography, Paper, Slider, Stack, Divider, Select, MenuItem } from '@mui/material';
import Grid from '@mui/material/Grid'; // Ensure you are importing standard Grid
import { WebRtcPlayer } from '../stream/WebRtcPlayer';

export const DashboardView = () => {
  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', p: 2 }}>
      
      {/* We use a 12-column grid. 
        Left Side: size 5 (41.6% width) - mirrors the wide tuning area in Limelight.
        Right Side: size 7 (58.3% width).
      */}
      <Grid container spacing={3} sx={{ flexGrow: 1, height: '100%' }}>
        
        {/* LEFT COLUMN - Wide Tuning Panel */}
        <Grid 
          size={{ xs: 12, md: 5 }} 
          sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}
        >
          <Stack spacing={3} sx={{ flexGrow: 1, height: '100%' }}>
            
            <Paper sx={{ 
              p: 4, 
              bgcolor: 'background.paper', 
              borderRadius: 2, 
              flexGrow: 1, 
              display: 'flex', 
              flexDirection: 'column' 
            }}>
              <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', fontSize: '0.85rem' }}>
                Input Processing
              </Typography>
              <Divider sx={{ my: 2 }} />
              
              <Stack spacing={4} sx={{ mt: 2 }}>
                <Box>
                  <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700, mb: 1, display: 'block' }}>EXPOSURE</Typography>
                  <Slider defaultValue={20} size="medium" />
                </Box>
                <Box>
                  <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700, mb: 1, display: 'block' }}>BRIGHTNESS</Typography>
                  <Slider defaultValue={50} size="medium" />
                </Box>
                <Box>
                  <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700, mb: 1, display: 'block' }}>PIPELINE</Typography>
                  <Select fullWidth size="small" defaultValue={0}>
                    <MenuItem value={0}>AprilTags 36h11</MenuItem>
                    <MenuItem value={1}>Neural Detector</MenuItem>
                  </Select>
                </Box>
              </Stack>
            </Paper>

            <Paper sx={{ p: 3, bgcolor: 'background.paper', borderRadius: 2 }}>
              <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main' }}>Sorting Logic</Typography>
              <Divider sx={{ my: 1.5 }} />
              <Select fullWidth size="small" defaultValue="largest">
                <MenuItem value="largest">Largest Area</MenuItem>
                <MenuItem value="closest">Closest to Center</MenuItem>
              </Select>
            </Paper>

          </Stack>
        </Grid>

        {/* RIGHT COLUMN - Video & Telemetry */}
        <Grid 
          size={{ xs: 12, md: 7 }} 
          sx={{ display: 'flex', flexDirection: 'column', height: '100%' }}
        >
          <Stack spacing={3} sx={{ height: '100%', flexGrow: 1 }}>
            
            {/* Video Card - No flexGrow to let it sit at its natural 16:9 ratio */}
            <Paper sx={{ 
              bgcolor: '#000', 
              borderRadius: 2, 
              overflow: 'hidden',
              border: '1px solid',
              borderColor: 'divider',
              lineHeight: 0
            }}>
               <WebRtcPlayer />
            </Paper>

            {/* Telemetry Card - Expands to fill the remaining vertical height */}
            <Paper sx={{ 
              p: 2, 
              bgcolor: 'background.paper', 
              borderRadius: 2, 
              flexGrow: 1, 
              display: 'flex', 
              flexDirection: 'column' 
            }}>
              <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', px: 1 }}>
                Target Data
              </Typography>
              <Divider sx={{ my: 1 }} />
              
              <Stack spacing={0.5} sx={{ mt: 1 }}>
                {/* Header Row (Optional, for clarity) */}
                <Box sx={{ display: 'flex', px: 2, pb: 1 }}>
                   <Typography variant="caption" sx={{ flex: 1, fontWeight: 700, color: 'text.disabled' }}>TARGET ID</Typography>
                   <Typography variant="caption" sx={{ flex: 1, fontWeight: 700, color: 'text.disabled', textAlign: 'center' }}>TX / TY</Typography>
                   <Typography variant="caption" sx={{ flex: 1, fontWeight: 700, color: 'text.disabled', textAlign: 'center' }}>AREA</Typography>
                   <Typography variant="caption" sx={{ flex: 1, fontWeight: 700, color: 'text.disabled', textAlign: 'right' }}>LATENCY</Typography>
                </Box>

                {/* Target Rows - Repeating list style */}
                {[
                  { id: '0', tx: '-14.25', ty: '+4.23', ta: '5.52', tl: '12ms', active: true },
                  { id: '1', tx: '+2.10', ty: '-1.45', ta: '1.20', tl: '14ms', active: false },
                  { id: '2', tx: '--', ty: '--', ta: '--', tl: '--', active: false },
                ].map((target) => (
                  <Box 
                    key={target.id}
                    sx={{ 
                      display: 'flex', 
                      p: 1.5, 
                      px: 2,
                      bgcolor: target.active ? 'rgba(255, 255, 0, 0.05)' : 'transparent',
                      border: '1px solid',
                      borderColor: target.active ? 'primary.main' : 'transparent',
                      borderRadius: 1,
                      alignItems: 'center'
                    }}
                  >
                    <Typography sx={{ flex: 1, fontWeight: 800, color: target.active ? 'primary.main' : 'text.secondary', fontFamily: 'monospace' }}>
                      #{target.id}
                    </Typography>
                    <Typography sx={{ flex: 1, textAlign: 'center', fontFamily: 'monospace', fontSize: '0.9rem' }}>
                      {target.tx}° / {target.ty}°
                    </Typography>
                    <Typography sx={{ flex: 1, textAlign: 'center', fontFamily: 'monospace', fontSize: '0.9rem' }}>
                      {target.ta}%
                    </Typography>
                    <Typography sx={{ flex: 1, textAlign: 'right', fontFamily: 'monospace', fontSize: '0.9rem', color: 'text.secondary' }}>
                      {target.tl}
                    </Typography>
                  </Box>
                ))}
              </Stack>
            </Paper>

          </Stack>
        </Grid>
      </Grid>
    </Box>
  );
};