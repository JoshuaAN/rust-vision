// src/theme.ts
import { createTheme } from '@mui/material/styles';

export const carbonTheme = createTheme({
  palette: {
    mode: 'dark',
    background: {
      default: '#171717',
      paper: '#262626',
    },
    primary: {
      main: '#facc15',
    },
    text: {
      primary: '#f5f5f5',
      secondary: '#a3a3a3',
    },
    divider: '#525252',
  },
  components: {
    MuiListItemButton: {
      styleOverrides: {
        root: {
          '&:hover': {
            backgroundColor: '#404040',
          },
        },
      },
    },
  },
});