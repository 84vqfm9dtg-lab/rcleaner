import FolderRoundedIcon from "@mui/icons-material/FolderRounded";
import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import MonitorWeightRoundedIcon from "@mui/icons-material/MonitorWeightRounded";
import ShieldRoundedIcon from "@mui/icons-material/ShieldRounded";
import { Divider, Paper, Stack, Typography } from "@mui/material";
import { formatPercent, riskLabel, riskPalette } from "../lib/format";
import type { ScanReport, TargetScan } from "../lib/types";

interface InspectorPanelProps {
  scanReport: ScanReport | null;
  activeTarget: TargetScan | null;
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <Stack
      direction="row"
      spacing={1}
      sx={{ justifyContent: "space-between", alignItems: "center" }}
    >
      <Typography variant="body2" sx={{ color: "text.secondary" }}>
        {label}
      </Typography>
      <Typography
        variant="body2"
        sx={{ fontWeight: 600, textAlign: "right", wordBreak: "break-word", maxWidth: "62%" }}
      >
        {value}
      </Typography>
    </Stack>
  );
}

export function InspectorPanel({ scanReport, activeTarget }: InspectorPanelProps) {
  const tone = activeTarget ? riskPalette(activeTarget.risk) : null;

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
            概览
          </Typography>
        </div>

        {scanReport ? (
          <Paper
            elevation={0}
            sx={{
              p: 1.05,
              borderRadius: "10px",
              bgcolor: "rgba(255,255,255,0.03)",
              border: "1px solid rgba(255,255,255,0.06)",
            }}
          >
            <Stack spacing={0.7}>
              <Stack direction="row" spacing={0.8} sx={{ alignItems: "center" }}>
                <MonitorWeightRoundedIcon fontSize="small" sx={{ color: "text.secondary" }} />
                <Typography variant="body2" sx={{ color: "text.secondary" }}>
                  资源盘概览
                </Typography>
              </Stack>
              <DetailRow label="总容量" value={scanReport.disk.totalLabel} />
              <DetailRow label="已使用" value={scanReport.disk.usedLabel} />
              <DetailRow label="可用" value={scanReport.disk.availableLabel} />
              <DetailRow label="占用比例" value={formatPercent(scanReport.disk.usedRatio)} />
            </Stack>
          </Paper>
        ) : null}

        <Divider />

        {activeTarget ? (
          <Stack spacing={0.95}>
            <Typography variant="subtitle1" sx={{ fontWeight: 700 }}>
              {activeTarget.title}
            </Typography>
            <Stack direction="row" spacing={0.65}>
              <Paper
                elevation={0}
                sx={{
                  px: 0.8,
                  py: 0.45,
                  borderRadius: "8px",
                  bgcolor: tone?.bg ?? "rgba(255,255,255,0.05)",
                  color: tone?.color ?? "#f2f0ea",
                  border: "1px solid",
                  borderColor: tone?.border ?? "rgba(255,255,255,0.08)",
                }}
              >
                <Typography variant="caption">{riskLabel(activeTarget.risk)}</Typography>
              </Paper>
              <Paper
                elevation={0}
                sx={{
                  px: 0.8,
                  py: 0.45,
                  borderRadius: "8px",
                  bgcolor: "rgba(255,255,255,0.05)",
                  border: "1px solid rgba(255,255,255,0.08)",
                }}
              >
                <Typography variant="caption">{activeTarget.category}</Typography>
              </Paper>
            </Stack>

            <Paper
              elevation={0}
              sx={{
                p: 0.95,
                borderRadius: "10px",
                bgcolor: "rgba(255,255,255,0.03)",
                border: "1px solid rgba(255,255,255,0.06)",
              }}
            >
              <Stack spacing={0.7}>
                <DetailRow label="当前体积" value={activeTarget.sizeLabel} />
                <DetailRow label="默认建议" value={activeTarget.defaultSelected ? "是" : "否"} />
                <DetailRow label="路径存在" value={activeTarget.exists ? "是" : "否"} />
              </Stack>
            </Paper>

            <Stack direction="row" spacing={0.7} sx={{ alignItems: "flex-start" }}>
              <FolderRoundedIcon fontSize="small" sx={{ color: "text.secondary", mt: 0.1 }} />
              <Typography
                variant="body2"
                sx={{
                  color: "text.secondary",
                  wordBreak: "break-word",
                  fontFamily:
                    '"SF Mono", "JetBrains Mono", "Fira Code", "Menlo", "Consolas", monospace',
                }}
              >
                {activeTarget.path}
              </Typography>
            </Stack>

            <Stack direction="row" spacing={0.7} sx={{ alignItems: "flex-start" }}>
              <ShieldRoundedIcon fontSize="small" sx={{ color: "text.secondary", mt: 0.1 }} />
              <Typography variant="body2" sx={{ color: "text.secondary" }}>
                {activeTarget.cleanable ? "可清理" : activeTarget.expertCleanable ? "专家模式可清理" : "概览"}
              </Typography>
            </Stack>
          </Stack>
        ) : (
          <Stack direction="row" spacing={0.7} sx={{ alignItems: "flex-start" }}>
            <InfoOutlinedIcon fontSize="small" sx={{ color: "text.secondary", mt: 0.1 }} />
            <Typography variant="body2" sx={{ color: "text.secondary" }}>
              选择目标后查看详情。
            </Typography>
          </Stack>
        )}
      </Stack>
    </Paper>
  );
}
