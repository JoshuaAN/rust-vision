// src/components/views/SettingsView.tsx
import React, { useState } from 'react';
import { 
  Box, Typography, Paper, Stack, Divider, 
  TextField, Radio, RadioGroup, FormControlLabel, 
  Button, Alert 
} from '@mui/material';
import Grid from '@mui/material/Grid';
import SaveIcon from '@mui/icons-material/Save';
import RestartAltIcon from '@mui/icons-material/RestartAlt';
import PowerSettingsNewIcon from '@mui/icons-material/PowerSettingsNew';

export const SettingsView = () => {
  // State for networking
  const [teamNumber, setTeamNumber] = useState('9999');
  const [hostname, setHostname] = useState('lemonlantern');
  const [ipMode, setIpMode] = useState('static'); // 'dhcp' or 'static'
  
  // Static IP states
  const [ipAddress, setIpAddress] = useState('10.99.99.11');
  const [subnetMask, setSubnetMask] = useState('255.255.255.0');
  const [gateway, setGateway] = useState('10.99.99.1');

  return (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column', p: 2, gap: 2, overflowY: 'auto' }}>
      
      <Grid container spacing={3} sx={{ flexGrow: 1 }}>
        
        {/* LEFT COLUMN: Network Configuration */}
        <Grid size={{ xs: 12, md: 6 }}>
          <Paper sx={{ p: 4, bgcolor: 'background.paper', borderRadius: 2, display: 'flex', flexDirection: 'column', gap: 3 }}>
            <Box>
              <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', fontSize: '0.85rem' }}>
                Network Configuration
              </Typography>
              <Divider sx={{ mt: 1 }} />
            </Box>

            <Stack spacing={3}>
              {/* Basic Info */}
              <Box sx={{ display: 'flex', gap: 2 }}>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    TEAM NUMBER
                  </Typography>
                  <TextField 
                  fullWidth size="small" 
                  value={teamNumber} 
                  onChange={(e) => {
                    const onlyNumbers = e.target.value.replace(/[^0-9]/g, '');
                    setTeamNumber(onlyNumbers);
                  }}
                  slotProps={{ htmlInput: { inputMode: 'numeric', pattern: '[0-9]*' } }}
                />
                </Box>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    HOSTNAME
                  </Typography>
                  <TextField 
                    fullWidth size="small" 
                    value={hostname}
                    onChange={(e) => setHostname(e.target.value)}
                  />
                </Box>
              </Box>

              {/* IP Allocation Mode */}
              <Box>
                <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                  IP ASSIGNMENT
                </Typography>
                <RadioGroup 
                  row 
                  value={ipMode} 
                  onChange={(e) => setIpMode(e.target.value)}
                >
                  <FormControlLabel value="dhcp" control={<Radio size="small" />} label={<Typography variant="body2" sx={{ fontWeight: 600 }}>DHCP (Dynamic)</Typography>} />
                  <FormControlLabel value="static" control={<Radio size="small" />} label={<Typography variant="body2" sx={{ fontWeight: 600 }}>Static IP</Typography>} />
                </RadioGroup>
              </Box>

              {/* Static IP Fields */}
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, opacity: ipMode === 'dhcp' ? 0.5 : 1, pointerEvents: ipMode === 'dhcp' ? 'none' : 'auto' }}>
                <Box>
                  <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                    IP ADDRESS
                  </Typography>
                  <TextField 
                    fullWidth size="small" 
                    value={ipAddress}
                    onChange={(e) => setIpAddress(e.target.value)}
                  />
                </Box>
                <Box sx={{ display: 'flex', gap: 2 }}>
                  <Box sx={{ flex: 1 }}>
                    <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                      SUBNET MASK
                    </Typography>
                    <TextField 
                      fullWidth size="small" 
                      value={subnetMask}
                      onChange={(e) => setSubnetMask(e.target.value)}
                    />
                  </Box>
                  <Box sx={{ flex: 1 }}>
                    <Typography variant="caption" sx={{ fontWeight: 700, color: 'text.secondary', display: 'block', mb: 1 }}>
                      GATEWAY
                    </Typography>
                    <TextField 
                      fullWidth size="small" 
                      value={gateway}
                      onChange={(e) => setGateway(e.target.value)}
                    />
                  </Box>
                </Box>
              </Box>

              <Alert severity="info" sx={{ bgcolor: 'rgba(2, 136, 209, 0.1)', color: 'info.light', '& .MuiAlert-icon': { color: 'info.light' } }}>
                Network changes require a system reboot to take effect.
              </Alert>

              <Button 
                variant="contained" 
                startIcon={<SaveIcon />} 
                sx={{ py: 1.5, fontWeight: 800 }}
              >
                Apply Network Settings
              </Button>
            </Stack>
          </Paper>
        </Grid>

        {/* RIGHT COLUMN: System & Admin */}
        <Grid size={{ xs: 12, md: 6 }}>
          <Stack spacing={3}>
            
            {/* System Management Card */}
            <Paper sx={{ p: 4, bgcolor: 'background.paper', borderRadius: 2 }}>
              <Box>
                <Typography variant="overline" sx={{ fontWeight: 800, color: 'primary.main', fontSize: '0.85rem' }}>
                  System Management
                </Typography>
                <Divider sx={{ mt: 1, mb: 3 }} />
              </Box>
              
              <Stack spacing={2}>
                <Button 
                  variant="outlined" 
                  color="warning" 
                  startIcon={<RestartAltIcon />}
                  sx={{ justifyContent: 'flex-start', px: 3, py: 1.5, fontWeight: 700 }}
                >
                  Restart Vision Service
                </Button>
                
                <Button 
                  variant="outlined" 
                  color="error" 
                  startIcon={<PowerSettingsNewIcon />}
                  sx={{ justifyContent: 'flex-start', px: 3, py: 1.5, fontWeight: 700 }}
                >
                  Reboot Coprocessor
                </Button>
              </Stack>
            </Paper>

            {/* Hardware Info Card (Read Only) */}
            <Paper sx={{ p: 4, bgcolor: 'background.paper', borderRadius: 2, flexGrow: 1 }}>
              <Box>
                <Typography variant="overline" sx={{ fontWeight: 800, color: 'text.disabled', fontSize: '0.85rem' }}>
                  Hardware Information
                </Typography>
                <Divider sx={{ mt: 1, mb: 3 }} />
              </Box>
              
              <Stack spacing={1.5}>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" sx={{ color: 'text.secondary', fontWeight: 600 }}>MAC Address</Typography>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 700 }}>AA:BB:CC:DD:EE:FF</Typography>
                </Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" sx={{ color: 'text.secondary', fontWeight: 600 }}>LemonLantern Version</Typography>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 700 }}>v1.0.4-beta</Typography>
                </Box>
                <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                  <Typography variant="body2" sx={{ color: 'text.secondary', fontWeight: 600 }}>CPU Temperature</Typography>
                  <Typography variant="body2" sx={{ fontFamily: 'monospace', fontWeight: 700, color: 'warning.main' }}>48°C</Typography>
                </Box>
              </Stack>
            </Paper>

          </Stack>
        </Grid>

      </Grid>
    </Box>
  );
};