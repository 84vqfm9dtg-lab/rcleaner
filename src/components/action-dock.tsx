import AutoFixHighRoundedIcon from "@mui/icons-material/AutoFixHighRounded";
import CleaningServicesRoundedIcon from "@mui/icons-material/CleaningServicesRounded";
import DoneAllRoundedIcon from "@mui/icons-material/DoneAllRounded";
import LayersClearRoundedIcon from "@mui/icons-material/LayersClearRounded";
import ShieldRoundedIcon from "@mui/icons-material/ShieldRounded";
import { Button, Divider, FormControlLabel, Paper, Stack, Switch, Typography } from "@mui/material";

interface ActionDockProps {
  selectedCount: number;
  selectedBytesLabel: string;
  defaultCount: number;
  generatedAt: string;
  cleanBusy: boolean;
  allowReadonlyClean: boolean;
  onToggleAllowReadonlyClean: () => void;
  onCleanSelected: () => void;
  onSelectDefaults: () => void;
  onClearSelection: () => void;
}

export function ActionDock({
  selectedCount,
  selectedBytesLabel,
  defaultCount,
  generatedAt,
  cleanBusy,
  allowReadonlyClean,
  onToggleAllowReadonlyClean,
  onCleanSelected,
  onSelectDefaults,
  onClearSelection,
}: ActionDockProps) {
  return (
    <Paper
      elevation={0}
      sx={{
        p: 1.2,
        borderRadius: "12px",
        border: "1px solid rgba(255,255,255,0.08)",
        bgcolor: "rgba(18,20,25,0.84)",
        boxShadow: "0 18px 36px rgba(0,0,0,0.18), inset 0 1px 0 rgba(255,255,255,0.04)",
      }}
    >
      <Stack spacing={1.2}>
        <div>
          <Typography variant="h6" sx={{ fontWeight: 700 }}>
            操作
          </Typography>
        </div>

        <Paper
          elevation={0}
          sx={{
            p: 1,
            borderRadius: "10px",
            bgcolor: "rgba(255,255,255,0.03)",
            border: "1px solid rgba(255,255,255,0.06)",
          }}
        >
          <Stack spacing={0.45}>
            <Typography variant="body2" sx={{ color: "text.secondary" }}>
              当前选择
            </Typography>
            <Typography variant="h4" sx={{ fontWeight: 700, lineHeight: 1 }}>
              {selectedCount}
            </Typography>
            <Typography variant="body2" sx={{ color: "text.secondary" }}>
              {selectedBytesLabel}
            </Typography>
          </Stack>
        </Paper>

        <Stack spacing={0.8}>
          <FormControlLabel
            sx={{ ml: 0 }}
            control={
              <Switch
                size="small"
                checked={allowReadonlyClean}
                onChange={onToggleAllowReadonlyClean}
                disabled={cleanBusy}
              />
            }
            label={allowReadonlyClean ? "专家模式（含只读）" : "安全模式"}
          />
          <Button
            variant="contained"
            startIcon={<CleaningServicesRoundedIcon />}
            onClick={onCleanSelected}
            disabled={cleanBusy || selectedCount === 0}
          >
            {cleanBusy ? "清理中" : "清理已选"}
          </Button>
          <Button
            variant="outlined"
            startIcon={<AutoFixHighRoundedIcon />}
            onClick={onSelectDefaults}
            disabled={cleanBusy || defaultCount === 0}
          >
            建议项
          </Button>
          <Button
            variant="outlined"
            color="inherit"
            startIcon={<LayersClearRoundedIcon />}
            onClick={onClearSelection}
            disabled={cleanBusy || selectedCount === 0}
          >
            清空选择
          </Button>
        </Stack>

        <Divider />
        <Stack direction="row" spacing={0.7} sx={{ alignItems: "center" }}>
          <DoneAllRoundedIcon fontSize="small" sx={{ color: "text.secondary" }} />
          <ShieldRoundedIcon fontSize="small" sx={{ color: "text.secondary" }} />
          <Typography variant="caption" sx={{ color: "text.secondary" }}>
            建议 {defaultCount} · {generatedAt}
          </Typography>
        </Stack>
      </Stack>
    </Paper>
  );
}
