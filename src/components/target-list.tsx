import CleaningServicesRoundedIcon from "@mui/icons-material/CleaningServicesRounded";
import FolderOpenRoundedIcon from "@mui/icons-material/FolderOpenRounded";
import WarningAmberRoundedIcon from "@mui/icons-material/WarningAmberRounded";
import {
  Box,
  Button,
  Checkbox,
  Chip,
  Paper,
  Stack,
  Typography,
} from "@mui/material";
import { riskLabel, riskPalette } from "../lib/format";
import type { TargetScan } from "../lib/types";

interface TargetListProps {
  items: TargetScan[];
  selectedIds: string[];
  activeId: string;
  cleanBusy: boolean;
  allowReadonlyClean: boolean;
  onFocus: (id: string) => void;
  onToggleSelect: (id: string) => void;
  onCleanSingle: (id: string) => void;
  onReveal: (path: string) => void;
}

export function TargetList({
  items,
  selectedIds,
  activeId,
  cleanBusy,
  allowReadonlyClean,
  onFocus,
  onToggleSelect,
  onCleanSingle,
  onReveal,
}: TargetListProps) {
  const sortedItems = [...items].sort((a, b) => {
    if (a.exists !== b.exists) {
      return a.exists ? -1 : 1;
    }
    return b.sizeBytes - a.sizeBytes;
  });

  const groupedItems = sortedItems.reduce(
    (map, item) => {
      const current = map.get(item.category) ?? [];
      current.push(item);
      map.set(item.category, current);
      return map;
    },
    new Map<string, TargetScan[]>(),
  );

  const groupEntries = Array.from(groupedItems.entries()).sort((a, b) => {
    const aBytes = a[1].reduce((sum, item) => sum + item.sizeBytes, 0);
    const bBytes = b[1].reduce((sum, item) => sum + item.sizeBytes, 0);
    return bBytes - aBytes;
  });

  if (groupEntries.length === 0) {
    return (
      <Paper
        elevation={0}
        sx={{
          p: 1.2,
          borderRadius: "10px",
          border: "1px solid rgba(255,255,255,0.08)",
          bgcolor: "rgba(21,24,30,0.82)",
        }}
      >
        <Typography variant="body2" sx={{ color: "text.secondary" }}>
          当前筛选下暂无可显示目标。
        </Typography>
      </Paper>
    );
  }

  return (
    <Stack spacing={1.1}>
      {groupEntries.map(([category, targets]) => {
        const groupBytes = targets.reduce((sum, item) => sum + item.sizeBytes, 0);
        return (
          <Paper
            key={category}
            elevation={0}
            sx={{
              p: 1,
              borderRadius: "10px",
              border: "1px solid rgba(255,255,255,0.08)",
              bgcolor: "rgba(18,22,28,0.62)",
            }}
          >
            <Stack spacing={0.9}>
              <Stack direction="row" spacing={0.8} sx={{ justifyContent: "space-between", alignItems: "center" }}>
                <Chip label={compactCategoryLabel(category)} size="small" title={category} />
                <Typography variant="caption" sx={{ color: "text.secondary" }}>
                  {targets.length} 项 · {formatBytes(groupBytes)}
                </Typography>
              </Stack>

              <Stack spacing={0.8}>
                {targets.map((item) => {
        const active = item.id === activeId;
        const checked = selectedIds.includes(item.id);
        const tone = riskPalette(item.risk);

        return (
          <Paper
            key={item.id}
            elevation={0}
            onClick={() => onFocus(item.id)}
            sx={{
              p: 1.15,
              borderRadius: "10px",
              cursor: "pointer",
              border: "1px solid",
              borderColor: active ? "rgba(255,255,255,0.18)" : "rgba(255,255,255,0.08)",
              bgcolor: active ? "rgba(255,255,255,0.055)" : "rgba(21,24,30,0.86)",
              boxShadow: active
                ? "0 22px 40px rgba(0,0,0,0.22), inset 0 1px 0 rgba(255,255,255,0.06)"
                : "inset 0 1px 0 rgba(255,255,255,0.035)",
              transition: "border-color 160ms ease, background-color 160ms ease",
            }}
          >
            <Stack spacing={0.9}>
              <Stack
                direction={{ xs: "column", sm: "row" }}
                spacing={0.8}
                sx={{ justifyContent: "space-between" }}
              >
                <Stack
                  direction="row"
                  spacing={0.8}
                  sx={{ alignItems: "center", minWidth: 0 }}
                  onClick={(event) => {
                    event.stopPropagation();
                    onToggleSelect(item.id);
                  }}
                >
                  <Checkbox
                    checked={checked}
                    disabled={!item.cleanable && !(allowReadonlyClean && item.expertCleanable)}
                    onClick={(event) => {
                      event.stopPropagation();
                      onToggleSelect(item.id);
                    }}
                    size="small"
                  />
                  <Box sx={{ minWidth: 0 }}>
                    <Typography variant="h6" sx={{ fontWeight: 700, lineHeight: 1.05 }}>
                      {item.title}
                    </Typography>
                </Box>
              </Stack>
                <Stack
                  direction="row"
                  spacing={0.65}
                  sx={{ alignItems: "center", flexWrap: "wrap" }}
                >
                  <Chip
                    label={riskLabel(item.risk)}
                    size="small"
                    sx={{
                      bgcolor: tone.bg,
                      color: tone.color,
                      border: "1px solid",
                      borderColor: tone.border,
                    }}
                  />
                  <Chip
                    label={item.sizeLabel}
                    size="small"
                    sx={{
                      bgcolor: "rgba(255,255,255,0.05)",
                      color: "#f2f0ea",
                    }}
                  />
                  {!item.cleanable ? (
                    <Chip
                      label={item.expertCleanable ? "只读" : "概览"}
                      size="small"
                      sx={{
                        bgcolor: "rgba(255,255,255,0.04)",
                        color: "text.secondary",
                        border: "1px solid rgba(255,255,255,0.08)",
                      }}
                    />
                  ) : null}
                </Stack>
              </Stack>

              <Typography
                variant="body2"
                sx={{
                  color: "text.secondary",
                  fontFamily:
                    '"SF Mono", "JetBrains Mono", "Fira Code", "Menlo", "Consolas", monospace',
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap",
                }}
                title={item.path}
              >
                {item.path}
              </Typography>

              {item.scanError ? (
                <Stack direction="row" spacing={0.7} sx={{ alignItems: "center" }}>
                  <WarningAmberRoundedIcon sx={{ color: "#e3c489", fontSize: 18 }} />
                  <Typography variant="body2" sx={{ color: "#e7d6b8" }}>
                    {item.scanError}
                  </Typography>
                </Stack>
              ) : null}

              <Stack
                direction="row"
                spacing={0.8}
                sx={{ justifyContent: "space-between", alignItems: "center" }}
              >
                <Typography
                  variant="caption"
                  sx={{ color: "text.secondary", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
                  title={item.exists ? item.caution : "当前路径不存在"}
                >
                  {item.exists ? "可打开目录查看详情" : "路径不存在"}
                </Typography>
                <Stack direction="row" spacing={0.75}>
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<FolderOpenRoundedIcon />}
                    onClick={(event) => {
                      event.stopPropagation();
                      onReveal(item.path);
                    }}
                  >
                    打开
                  </Button>
                  <Button
                    variant="contained"
                    size="small"
                    startIcon={<CleaningServicesRoundedIcon />}
                    disabled={
                      cleanBusy || (!item.cleanable && !(allowReadonlyClean && item.expertCleanable))
                    }
                    onClick={(event) => {
                      event.stopPropagation();
                      onCleanSingle(item.id);
                    }}
                  >
                    {cleanBusy && (item.cleanable || (allowReadonlyClean && item.expertCleanable))
                      ? "清理中"
                      : item.cleanable || (allowReadonlyClean && item.expertCleanable)
                        ? "清理"
                        : "概览"}
                  </Button>
                </Stack>
              </Stack>
            </Stack>
          </Paper>
        );
      })}
              </Stack>
            </Stack>
          </Paper>
        );
      })}
    </Stack>
  );
}

function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${Math.round(value)} ${units[unit]}` : `${value.toFixed(1)} ${units[unit]}`;
}

function compactCategoryLabel(label: string): string {
  const map: Record<string, string> = {
    "应用容器（只读）": "应用容器",
    "应用支持数据（只读）": "支持数据",
    "应用缓存（可清理）": "应用缓存",
    "AI 开发工具（只读）": "AI 工具",
  };
  return map[label] ?? label;
}
