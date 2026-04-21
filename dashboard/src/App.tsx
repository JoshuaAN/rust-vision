import { useState } from 'react';
import { Sidebar } from './components/layout/Sidebar';
import { ThemeProvider, CssBaseline, Box } from '@mui/material';
import { carbonTheme } from './theme';

// Import your views (you'll create these files next)
// For now, I'll define placeholders below so the code runs immediately
import { DashboardView } from './components/views/Dashboard'; 

export default function App() {
  // Use the labels from your Sidebar to control which view is rendered
  const [activeTab, setActiveTab] = useState('Live Stream');

  // Logic to determine which component to show
  const renderView = () => {
    switch (activeTab) {
      case 'Live Stream':
        return <DashboardView />;
      case 'Camera':
        return <CameraViewPlaceholder />;
      case 'Settings':
        return <SettingsViewPlaceholder />;
      default:
        return <DashboardView />;
    }
  };

  return (
    <ThemeProvider theme={carbonTheme}>
      <CssBaseline /> 
      <Box sx={{ display: 'flex', height: '100vh', bgcolor: 'background.default' }}>
        
        {/* Pass state and setter to Sidebar so it can change the view */}
        <Sidebar activeTab={activeTab} setActiveTab={setActiveTab} />
        
        <Box 
  component="main" 
  sx={{ 
    flexGrow: 1, 
    height: '100vh',
    p: 3, 
    overflow: 'hidden',
    display: 'flex',      // Add this
    flexDirection: 'column' // Add this
  }}
>
  {renderView()}
</Box>
      </Box>
    </ThemeProvider>
  );
}

// --- Temporary Placeholders ---
// You can move these into their own files in src/components/views/
const CameraViewPlaceholder = () => (
  <Box>
    <Box sx={{ color: 'text.primary', typography: 'h5', fontWeight: 800, mb: 2 }}>
      Camera Calibration
    </Box>
    <Box sx={{ color: 'text.secondary' }}>Calibration and intrinsics configuration will go here.</Box>
  </Box>
);

const SettingsViewPlaceholder = () => (
  <Box>
    <Box sx={{ color: 'text.primary', typography: 'h5', fontWeight: 800, mb: 2 }}>
      Network Settings
    </Box>
    <Box sx={{ color: 'text.secondary' }}>Networking and system administration will go here.</Box>
  </Box>
);