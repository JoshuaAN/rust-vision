import { useState } from 'react';
import { Sidebar } from './components/layout/Sidebar';
import { ThemeProvider, CssBaseline, Box } from '@mui/material';
import { carbonTheme } from './theme';

// Import your views (you'll create these files next)
// For now, I'll define placeholders below so the code runs immediately
import { DashboardView } from './components/views/Dashboard'; 
import { CameraView } from './components/views/Camera';
import { SettingsView } from './components/views/Settings';
import { CalibrationView } from './components/views/Calibration';

export default function App() {
  // Use the labels from your Sidebar to control which view is rendered
  const [activeTab, setActiveTab] = useState('Live Stream');

  // Logic to determine which component to show
  const renderView = () => {
    switch (activeTab) {
      case 'Live Stream':
        return <DashboardView />;
      case 'Camera':
        return <CameraView />;
      case 'Calibration':
        return <CalibrationView />;
      case 'Settings':
        return <SettingsView />;
      default:
        return <DashboardView />;
    }
  };

  return (
    <ThemeProvider theme={carbonTheme}>
      <CssBaseline /> 
      <Box sx={{ display: 'flex', height: '100vh', bgcolor: 'background.default' }}>
        
        {/* Pass state and setter to Sidebar so it can change the view */}
        <Sidebar activeTab={activeTab} setActiveTab={setActiveTab}  isConnected={false}/>
        
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