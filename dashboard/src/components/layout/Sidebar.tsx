// src/components/Sidebar.tsx
import React, { useState } from 'react';
import { 
  Box, Typography, List, ListItem, ListItemButton, ListItemIcon, ListItemText
} from '@mui/material';
import VideocamIcon from '@mui/icons-material/Videocam';
import CameraAltIcon from '@mui/icons-material/CameraAlt';
import SettingsIcon from '@mui/icons-material/Settings';
import HighlightIcon from '@mui/icons-material/Highlight';

interface SidebarProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
}

export const Sidebar = ({ activeTab, setActiveTab }: SidebarProps) => {
  const menuItems = [
    { label: 'Live Stream', icon: <VideocamIcon /> },
    { label: 'Camera', icon: <CameraAltIcon /> },
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
    </Box>
  );
};