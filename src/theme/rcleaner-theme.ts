import { createTheme } from "@mui/material/styles";

const fontFamily = [
  '"SF Pro Display"',
  '"Avenir Next"',
  '"PingFang SC"',
  '"Hiragino Sans GB"',
  '"Microsoft YaHei"',
  "sans-serif",
].join(", ");

export const rcleanerTheme = createTheme({
  shape: {
    borderRadius: 8,
  },
  typography: {
    fontFamily,
    button: {
      textTransform: "none",
      fontWeight: 600,
      letterSpacing: 0,
    },
  },
  palette: {
    mode: "dark",
    primary: {
      main: "#f0ede8",
      contrastText: "#111216",
    },
    secondary: {
      main: "#9aa5b4",
    },
    background: {
      default: "#0c0e12",
      paper: "#15181d",
    },
    text: {
      primary: "#f3f1ec",
      secondary: "#9aa3ad",
    },
    divider: "rgba(255,255,255,0.08)",
  },
  components: {
    MuiCssBaseline: {
      styleOverrides: {
        body: {
          background:
            "radial-gradient(circle at top, rgba(255,255,255,0.05), transparent 30%), #0c0e12",
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundImage: "none",
        },
      },
    },
    MuiButton: {
      defaultProps: {
        disableElevation: true,
      },
      styleOverrides: {
        root: {
          minHeight: 36,
          borderRadius: 8,
          paddingInline: 12,
        },
      },
    },
    MuiChip: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          height: 30,
        },
      },
    },
    MuiOutlinedInput: {
      styleOverrides: {
        root: {
          borderRadius: 8,
          backgroundColor: "rgba(255,255,255,0.03)",
        },
      },
    },
  },
});
