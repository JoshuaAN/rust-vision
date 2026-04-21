// src/components/Sidebar.tsx
import { 
  Box, Typography, List, ListItem, ListItemButton, ListItemIcon, ListItemText
} from '@mui/material';
import VideocamIcon from '@mui/icons-material/Videocam';
import CameraAltIcon from '@mui/icons-material/CameraAlt';
import SettingsIcon from '@mui/icons-material/Settings';
import HighlightIcon from '@mui/icons-material/Highlight';
import CenterFocusStrongIcon from '@mui/icons-material/CenterFocusStrong';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  isConnected: boolean;
}

export const Sidebar = ({ activeTab, setActiveTab, isConnected = true }: SidebarProps) => {
  const menuItems = [
    { label: 'Live Stream', icon: <VideocamIcon /> },
    { label: 'Camera', icon: <CameraAltIcon /> },
    { label: 'Calibration', icon: <CenterFocusStrongIcon /> },
    { label: 'Settings', icon: <SettingsIcon /> },
  ];

  return (
    <Box
      component="aside"
      sx={{
        width: 260,
        height: '100vh',
        bgcolor: 'background.paper', 
        borderRight: 1,
        borderColor: 'divider', 
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <Box sx={{ height: 72, display: 'flex', alignItems: 'center', px: 3, gap: 1.5, borderBottom: 1, borderColor: 'divider' }}>
        <HighlightIcon color="primary" sx={{ fontSize: 28 }} />
        <Typography variant="h6" sx={{ fontWeight: 800, color: 'text.primary' }}>
          LemonLantern
        </Typography>
      </Box>

      <List sx={{ flexGrow: 1, px: 2, py: 3, display: 'flex', flexDirection: 'column', gap: 1 }}>
        {menuItems.map((item) => {
          const isActive = activeTab === item.label;
          
          return (
            <ListItem key={item.label} disablePadding>
              <ListItemButton
                onClick={() => setActiveTab(item.label)}
                sx={{
                  borderRadius: 2,
                  bgcolor: isActive ? 'rgba(250, 204, 21, 0.1)' : 'transparent',
                  '&:hover': {
                    bgcolor: isActive ? 'rgba(250, 204, 21, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                  },
                }}
              >
                <ListItemIcon sx={{ 
                  minWidth: 40, 
                  color: isActive ? 'primary.main' : 'text.secondary' 
                }}>
                  {item.icon}
                </ListItemIcon>
                
                <ListItemText 
                  primary={item.label} 
                  sx={{
                    '& .MuiListItemText-primary': {
                      fontWeight: isActive ? 600 : 500, 
                      color: isActive ? 'primary.main' : 'text.secondary',
                      fontSize: '0.9rem',
                    },
                  }} 
                />
              </ListItemButton>
            </ListItem>
          );
        })}
      </List>
      {/* CONNECTION STATUS FOOTER */}
      <Box sx={{ 
        p: 3, 
        borderTop: 1, 
        borderColor: 'divider', 
        display: 'flex', 
        alignItems: 'center', 
        gap: 2,
        bgcolor: 'rgba(0,0,0,0.1)' // Slight darkening for the footer area
      }}>
        {/* Status Dot */}
        <Box
          sx={{
            width: 12,
            height: 12,
            borderRadius: '50%',
            bgcolor: isConnected ? 'success.main' : 'error.main',
            boxShadow: isConnected ? '0 0 8px rgba(76, 175, 80, 0.5)' : '0 0 8px rgba(244, 67, 54, 0.5)',
            flexShrink: 0
          }}
        />
        
        {/* Status Text */}
        <Box>
          <Typography variant="caption" sx={{ color: 'text.secondary', fontWeight: 700, display: 'block', lineHeight: 1 }}>
            SYSTEM STATUS
          </Typography>
          <Typography variant="body2" sx={{ 
            color: isConnected ? 'text.primary' : 'error.main', 
            fontWeight: 800, 
            mt: 0.5 
          }}>
            {isConnected ? 'Connected' : 'Disconnected'}
          </Typography>
        </Box>
      </Box>
    </Box>
  );
};