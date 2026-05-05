import AutoAwesomeIcon from "@mui/icons-material/AutoAwesome";
import RefreshRoundedIcon from "@mui/icons-material/RefreshRounded";
import StorageRoundedIcon from "@mui/icons-material/StorageRounded";
import { Box, Button, Chip, LinearProgress, Paper, Stack, Typography } from "@mui/material";

interface AppTopbarProps {
  reclaimableLabel: string;
  availableLabel: string;
  usedRatio: string;
  busy: boolean;
  onRefresh: () => void;
}

export function AppTopbar({
  reclaimableLabel,
  availableLabel,
  usedRatio,
  busy,
  onRefresh,
}: AppTopbarProps) {
  return (
    <Paper
      elevation={0}
      sx={{
        px: { xs: 1.2, sm: 1.5 },
        py: { xs: 0.95, sm: 1.05 },
        borderRadius: "12px",
        border: "1px solid rgba(255,255,255,0.08)",
        bgcolor: "rgba(20,23,29,0.82)",
        backdropFilter: "blur(18px)",
        boxShadow: "0 22px 46px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.05)",
      }}
    >
      <Stack
        direction={{ xs: "column", lg: "row" }}
        spacing={1.1}
        sx={{
          justifyContent: "space-between",
          alignItems: { xs: "stretch", lg: "center" },
        }}
      >
        <Stack direction="row" spacing={1.2} sx={{ alignItems: "center" }}>
          <Box
            sx={{
              width: 50,
              height: 50,
              borderRadius: "10px",
              display: "grid",
              placeItems: "center",
              bgcolor: "#f1efe9",
              color: "#121317",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.78)",
              fontSize: "1.9rem",
              fontWeight: 700,
            }}
          >
            R
          </Box>
          <Box>
            <Typography variant="h5" sx={{ fontWeight: 700, lineHeight: 1.05, letterSpacing: "0.01em" }}>
              rCleaner
            </Typography>
            <Typography variant="caption" sx={{ color: "text.secondary" }}>
              资源管理
            </Typography>
          </Box>
        </Stack>

        <Stack
          direction={{ xs: "column", sm: "row" }}
          spacing={0.8}
          sx={{ alignItems: "stretch" }}
        >
          <Chip icon={<AutoAwesomeIcon />} label={`释放 ${reclaimableLabel}`} />
          <Chip icon={<StorageRoundedIcon />} label={`可用 ${availableLabel}`} />
          <Chip label={`${usedRatio}`} />
          <Button
            variant="contained"
            startIcon={<RefreshRoundedIcon />}
            onClick={onRefresh}
            disabled={busy}
          >
            {busy ? "扫描中" : "重新扫描"}
          </Button>
        </Stack>
      </Stack>
      {busy ? <LinearProgress sx={{ mt: 1.1, borderRadius: "6px" }} /> : null}
    </Paper>
  );
}
