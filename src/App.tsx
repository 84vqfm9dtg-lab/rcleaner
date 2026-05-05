import { useEffect, useMemo, useState } from "react";
import CleaningServicesRoundedIcon from "@mui/icons-material/CleaningServicesRounded";
import {
  Alert,
  Box,
  Button,
  Chip,
  Container,
  CircularProgress,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  FormControlLabel,
  Paper,
  Checkbox,
  Stack,
  ToggleButton,
  ToggleButtonGroup,
  Typography,
} from "@mui/material";
import { AppTopbar } from "./components/app-topbar";
import { ActionDock } from "./components/action-dock";
import { InspectorPanel } from "./components/inspector-panel";
import { TargetList } from "./components/target-list";
import "./App.css";
import { cleanTargets, revealPath, scanTargets } from "./lib/api";
import { formatTimestamp, formatPercent } from "./lib/format";
import type { CleanReport, ScanReport, TargetScan } from "./lib/types";

type FeedbackState = {
  severity: "success" | "info" | "warning" | "error";
  text: string;
};

type RiskFilter = "all" | "low" | "elevated";

function App() {
  const [scanReport, setScanReport] = useState<ScanReport | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [activeId, setActiveId] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("全部");
  const [riskFilter, setRiskFilter] = useState<RiskFilter>("all");
  const [confirmTargetIds, setConfirmTargetIds] = useState<string[] | null>(null);
  const [elevatedRiskAcknowledged, setElevatedRiskAcknowledged] = useState(false);
  const [allowReadonlyClean, setAllowReadonlyClean] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [isCleaning, setIsCleaning] = useState(false);
  const [feedback, setFeedback] = useState<FeedbackState>({
    severity: "info",
    text: "准备扫描。",
  });

  async function refreshTargets(
    preferDefaults = false,
    nextFeedback?: FeedbackState,
    options?: { silent?: boolean },
  ) {
    const silent = options?.silent ?? false;
    try {
      if (!silent) {
        setIsScanning(true);
      }
      const nextReport = await scanTargets();
      setScanReport(nextReport);
      setCategoryFilter((current) => {
        const categories = new Set(nextReport.targets.map((item) => item.category));
        if (current === "全部" || categories.has(current)) {
          return current;
        }
        return "全部";
      });
      setSelectedIds((current) => {
        const nextIds = new Set(nextReport.targets.map((item) => item.id));
        const preserved = current.filter((id) => nextIds.has(id));
        if (preferDefaults) {
          return nextReport.targets
            .filter((item) => item.defaultSelected && item.exists)
            .map((item) => item.id);
        }
        return preserved;
      });
      setActiveId((current) => {
        if (nextReport.targets.some((item) => item.id === current)) {
          return current;
        }
        return nextReport.targets[0]?.id ?? "";
      });
      setFeedback(
        nextFeedback ?? {
          severity: "success",
          text: `已扫描 ${nextReport.targetCount} 项 · 可释放 ${nextReport.reclaimableLabel}`,
        },
      );
    } catch (error) {
      setFeedback({
        severity: "error",
        text: error instanceof Error ? error.message : "资源扫描失败",
      });
    } finally {
      if (!silent) {
        setIsScanning(false);
      }
    }
  }

  useEffect(() => {
    void refreshTargets(true);
  }, []);

  const categoryOptions = useMemo(() => {
    if (!scanReport) {
      return ["全部"];
    }

    return ["全部", ...Array.from(new Set(scanReport.targets.map((item) => item.category)))];
  }, [scanReport]);

  const categoryCounts = useMemo(() => {
    const map = new Map<string, number>();
    map.set("全部", scanReport?.targets.length ?? 0);
    for (const item of scanReport?.targets ?? []) {
      map.set(item.category, (map.get(item.category) ?? 0) + 1);
    }
    return map;
  }, [scanReport]);

  const visibleTargets = useMemo(() => {
    const targets = scanReport?.targets ?? [];
    return targets
      .filter((item) => categoryFilter === "全部" || item.category === categoryFilter)
      .filter((item) => {
        if (riskFilter === "all") {
          return true;
        }
        if (riskFilter === "low") {
          return item.risk === "low";
        }
        return item.risk !== "low";
      })
      .sort((a, b) => {
        if (a.exists !== b.exists) {
          return a.exists ? -1 : 1;
        }
        return b.sizeBytes - a.sizeBytes;
      });
  }, [scanReport, categoryFilter, riskFilter]);

  useEffect(() => {
    if (visibleTargets.length === 0) {
      setActiveId("");
      return;
    }
    if (!visibleTargets.some((item) => item.id === activeId)) {
      setActiveId(visibleTargets[0].id);
    }
  }, [visibleTargets, activeId]);

  const activeTarget = useMemo<TargetScan | null>(
    () => visibleTargets.find((item) => item.id === activeId) ?? null,
    [activeId, visibleTargets],
  );

  const selectedTargets = useMemo(
    () => scanReport?.targets.filter((item) => selectedIds.includes(item.id)) ?? [],
    [scanReport, selectedIds],
  );

  const selectedBytes = useMemo(
    () => selectedTargets.reduce((sum, item) => sum + item.sizeBytes, 0),
    [selectedTargets],
  );

  const selectedBytesLabel = useMemo(() => {
    if (selectedTargets.length === 0) {
      return "0 B";
    }
    return selectedTargets[0].sizeLabel && selectedBytes === selectedTargets[0].sizeBytes
      ? selectedTargets[0].sizeLabel
      : scanReport
        ? formatBytes(selectedBytes)
        : "0 B";
  }, [scanReport, selectedBytes, selectedTargets]);

  const defaultCount = useMemo(
    () => scanReport?.targets.filter((item) => item.defaultSelected).length ?? 0,
    [scanReport],
  );

  const confirmTargets = useMemo(() => {
    if (!confirmTargetIds || !scanReport) {
      return [];
    }
    const idSet = new Set(confirmTargetIds);
    return scanReport.targets.filter((item) => idSet.has(item.id));
  }, [confirmTargetIds, scanReport]);

  const confirmBytes = useMemo(
    () => confirmTargets.reduce((sum, item) => sum + item.sizeBytes, 0),
    [confirmTargets],
  );

  const confirmHasElevatedRisk = useMemo(
    () => confirmTargets.some((item) => item.risk !== "low"),
    [confirmTargets],
  );
  const confirmHasReadonly = useMemo(
    () => confirmTargets.some((item) => !item.cleanable),
    [confirmTargets],
  );

  async function executeClean(targetIds: string[]) {
    if (targetIds.length === 0) {
      setFeedback({
        severity: "warning",
        text: "请先选择目标。",
      });
      return;
    }

    try {
      setIsCleaning(true);
      const includesReadonly = targetIds.some((id) =>
        scanReport?.targets.some((item) => item.id === id && !item.cleanable),
      );
      const report = await cleanTargets(targetIds, false, allowReadonlyClean || includesReadonly);
      const nextFeedback = buildCleanFeedback(report);
      applyCleanReport(report);
      setFeedback(nextFeedback);
      setIsCleaning(false);
      await refreshTargets(false, nextFeedback, { silent: true });
    } catch (error) {
      setFeedback({
        severity: "error",
        text: error instanceof Error ? error.message : "清理失败",
      });
    } finally {
      setIsCleaning(false);
    }
  }

  function buildCleanFeedback(report: CleanReport): FeedbackState {
    const failedCount = report.targets.filter((item) => !item.ok).length;
    const first = report.targets[0];
    if (report.targetCount === 1 && first) {
      return {
        severity: first.ok ? "success" : "warning",
        text: `${first.title} 已处理，释放 ${first.freedLabel}。`,
      };
    }

    if (failedCount > 0) {
      return {
        severity: "warning",
        text: `共处理 ${report.targetCount} 项，释放 ${report.freedLabel}，其中 ${failedCount} 项失败。`,
      };
    }

    return {
      severity: "success",
      text: `已处理 ${report.targetCount} 项，共释放 ${report.freedLabel}。`,
    };
  }

  function applyCleanReport(report: CleanReport) {
    const cleanedTargets = new Map(report.targets.filter((item) => item.ok).map((item) => [item.id, item]));
    if (cleanedTargets.size === 0) {
      return;
    }

    setSelectedIds((current) => current.filter((id) => !cleanedTargets.has(id)));
    setScanReport((current) => {
      if (!current) {
        return current;
      }

      const targets = current.targets.map((item) => {
        const cleaned = cleanedTargets.get(item.id);
        if (!cleaned) {
          return item;
        }
        return {
          ...item,
          sizeBytes: cleaned.afterBytes,
          sizeLabel: cleaned.afterLabel,
          scanError: null,
        };
      });
      const reclaimableBytes = targets
        .filter((item) => item.cleanable)
        .reduce((sum, item) => sum + item.sizeBytes, 0);

      return {
        ...current,
        reclaimableBytes,
        reclaimableLabel: formatBytes(reclaimableBytes),
        existingCount: targets.filter((item) => item.exists).length,
        targets,
      };
    });
  }

  function requestClean(targetIds: string[]) {
    const uniqueIds = Array.from(new Set(targetIds));
    if (uniqueIds.length === 0) {
      setFeedback({
        severity: "warning",
        text: "请先选择目标。",
      });
      return;
    }

    const acceptedIds = uniqueIds.filter((id) =>
      scanReport?.targets.some(
        (item) => item.id === id && (item.cleanable || (allowReadonlyClean && item.expertCleanable)),
      ),
    );
    const ignoredOverviewCount = uniqueIds.length - acceptedIds.length;

    if (acceptedIds.length === 0) {
      setFeedback({
        severity: "info",
        text: allowReadonlyClean ? "当前没有可处理目标。" : "当前选择均为只读目标。",
      });
      return;
    }

    if (ignoredOverviewCount > 0) {
      setFeedback({
        severity: "info",
        text: `已忽略 ${ignoredOverviewCount} 个概览目标。`,
      });
    }

    setConfirmTargetIds(acceptedIds);
    setElevatedRiskAcknowledged(false);
  }

  function isSelectableTarget(item: TargetScan): boolean {
    return item.cleanable || (allowReadonlyClean && item.expertCleanable);
  }

  async function handleConfirmClean() {
    if (!confirmTargetIds) {
      return;
    }
    const ids = [...confirmTargetIds];
    setConfirmTargetIds(null);
    setElevatedRiskAcknowledged(false);
    await executeClean(ids);
  }

  function toggleSelected(id: string) {
    const target = scanReport?.targets.find((item) => item.id === id);
    if (!target) {
      return;
    }

    if (!isSelectableTarget(target)) {
      setActiveId(id);
      setFeedback({
        severity: "info",
        text: target.expertCleanable
          ? "该目标需要开启专家模式后才能选择。"
          : "这是整体占用视图，里面可能包含用户数据。请打开查看，或清理列表里拆出的缓存子项。",
      });
      return;
    }

    setSelectedIds((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    );
    setActiveId(id);
  }

  function selectDefaults() {
    if (!scanReport) {
      return;
    }
    setSelectedIds(scanReport.targets.filter((item) => item.defaultSelected).map((item) => item.id));
    setFeedback({
      severity: "info",
      text: "已切换建议项。",
    });
  }

  function toggleReadonlyCleanMode() {
    setAllowReadonlyClean((current) => {
      const next = !current;
      if (!next) {
        setSelectedIds((existing) =>
          existing.filter((id) => scanReport?.targets.some((item) => item.id === id && item.cleanable)),
        );
        setFeedback({
          severity: "info",
          text: "已切回安全模式，残留候选将不参与清理。",
        });
      } else {
        setFeedback({
          severity: "warning",
          text: "已开启专家模式：残留候选可被选择和清理。",
        });
      }
      return next;
    });
  }

  async function handleReveal(path: string) {
    try {
      await revealPath(path);
    } catch (error) {
      setFeedback({
        severity: "error",
        text: error instanceof Error ? error.message : "打开 Finder 失败",
      });
    }
  }

  return (
    <Box
      sx={{
        minHeight: "100vh",
        py: { xs: 1.2, md: 1.6 },
      }}
    >
      <Container maxWidth={false} sx={{ px: { xs: 1.2, sm: 1.6 } }}>
        <Stack spacing={1.3}>
          <AppTopbar
            reclaimableLabel={scanReport?.reclaimableLabel ?? "0 B"}
            availableLabel={scanReport?.disk.availableLabel ?? "0 B"}
            usedRatio={scanReport ? formatPercent(scanReport.disk.usedRatio) : "0%"}
            busy={isScanning}
            onRefresh={() => void refreshTargets(false)}
          />

          <Box
            sx={{
              display: "grid",
              gap: 1.2,
              gridTemplateColumns: {
                xs: "1fr",
                lg: "280px minmax(0, 1fr) 320px",
              },
              alignItems: "start",
            }}
          >
            <ActionDock
              selectedCount={selectedIds.length}
              selectedBytesLabel={selectedBytesLabel}
              defaultCount={defaultCount}
              generatedAt={scanReport ? formatTimestamp(scanReport.generatedAt) : "未扫描"}
              cleanBusy={isCleaning}
              allowReadonlyClean={allowReadonlyClean}
              onToggleAllowReadonlyClean={toggleReadonlyCleanMode}
              onCleanSelected={() => requestClean(selectedIds)}
              onSelectDefaults={selectDefaults}
              onClearSelection={() => setSelectedIds([])}
            />

            <Paper
              elevation={0}
              sx={{
                p: 1.2,
                borderRadius: "12px",
                border: "1px solid rgba(255,255,255,0.08)",
                bgcolor: "rgba(18,20,25,0.82)",
                boxShadow: "0 22px 46px rgba(0,0,0,0.2), inset 0 1px 0 rgba(255,255,255,0.04)",
                display: "flex",
                flexDirection: "column",
              }}
            >
              <Stack spacing={1.05}>
                <Stack
                  direction={{ xs: "column", md: "row" }}
                  spacing={0.8}
                  sx={{
                    justifyContent: "space-between",
                    alignItems: { xs: "flex-start", md: "center" },
                  }}
                >
                  <Box>
                    <Typography variant="h5" sx={{ fontWeight: 700 }}>
                      资源目标
                    </Typography>
                  </Box>
                  <Button
                    variant="contained"
                    startIcon={
                      isCleaning ? (
                        <CircularProgress size={16} color="inherit" />
                      ) : (
                        <CleaningServicesRoundedIcon />
                      )
                    }
                    disabled={isCleaning || selectedIds.length === 0}
                    onClick={() => requestClean(selectedIds)}
                  >
                    {isCleaning ? "清理中" : "清理已选"}
                  </Button>
                </Stack>

                <Paper
                  elevation={0}
                  sx={{
                    px: 0.75,
                    pt: 0.7,
                    pb: 0.65,
                    borderRadius: "10px",
                    bgcolor: "rgba(255,255,255,0.025)",
                    border: "1px solid rgba(255,255,255,0.06)",
                  }}
                >
                  <Stack spacing={0.6}>
                    <Stack
                      direction="row"
                      sx={{
                        alignItems: "center",
                        columnGap: 0.55,
                        rowGap: 0.5,
                        flexWrap: "wrap",
                      }}
                    >
                      <Typography
                        variant="caption"
                        sx={{ color: "text.secondary", flex: "0 0 auto", minWidth: 32, mr: 0.15 }}
                      >
                        分类
                      </Typography>
                      {categoryOptions.map((option) => (
                        <Chip
                          key={option}
                          size="small"
                          clickable
                          title={option}
                          label={`${compactCategoryLabel(option)} ${categoryCounts.get(option) ?? 0}`}
                          color={categoryFilter === option ? "primary" : "default"}
                          variant={categoryFilter === option ? "filled" : "outlined"}
                          onClick={() => setCategoryFilter(option)}
                          sx={{
                            flex: "0 0 auto",
                            height: 26,
                            borderRadius: "8px",
                            fontSize: "0.78rem",
                            "& .MuiChip-label": { px: 0.8 },
                          }}
                        />
                      ))}
                    </Stack>

                    <Stack
                      direction={{ xs: "column", sm: "row" }}
                      spacing={0.65}
                      sx={{
                        alignItems: { xs: "flex-start", sm: "center" },
                        justifyContent: "space-between",
                        pt: 0.1,
                      }}
                    >
                      <Typography variant="caption" sx={{ color: "text.secondary" }}>
                        风险
                      </Typography>
                      <ToggleButtonGroup
                        exclusive
                        size="small"
                        value={riskFilter}
                        onChange={(_, value: RiskFilter | null) => {
                          if (value) {
                            setRiskFilter(value);
                          }
                        }}
                        sx={{
                          p: 0.25,
                          borderRadius: "9px",
                          bgcolor: "rgba(0,0,0,0.18)",
                          border: "1px solid rgba(255,255,255,0.06)",
                          "& .MuiToggleButton-root": {
                            minWidth: 48,
                            height: 26,
                            px: 1,
                            border: 0,
                            borderRadius: "7px !important",
                            color: "text.secondary",
                            fontWeight: 700,
                          },
                          "& .Mui-selected": {
                            bgcolor: "#f1efe9 !important",
                            color: "#15171c !important",
                          },
                        }}
                      >
                        <ToggleButton value="all">全部</ToggleButton>
                        <ToggleButton value="low">低</ToggleButton>
                        <ToggleButton value="elevated">中高</ToggleButton>
                      </ToggleButtonGroup>
                    </Stack>
                  </Stack>
                </Paper>

                <Alert
                  severity={feedback.severity}
                  sx={{
                    borderRadius: "10px",
                    bgcolor: "rgba(255,255,255,0.03)",
                    border: "1px solid rgba(255,255,255,0.06)",
                    color: "text.primary",
                    "& .MuiAlert-icon": {
                      color: "inherit",
                    },
                  }}
                >
                  {feedback.text}
                </Alert>

                {allowReadonlyClean ? (
                  <Alert severity="warning" sx={{ borderRadius: "10px" }}>
                    专家模式已开启：仅疑似残留候选可被选择；概览项仍只支持打开查看。
                  </Alert>
                ) : null}

                <Box
                  sx={{
                    minHeight: 0,
                    maxHeight: {
                      xs: "54vh",
                      md: "58vh",
                      lg: "calc(100vh - 300px)",
                    },
                    overflowY: "auto",
                    overflowX: "hidden",
                    pr: 0.45,
                    mr: -0.2,
                    scrollbarGutter: "stable",
                  }}
                >
                  <TargetList
                    items={visibleTargets}
                    selectedIds={selectedIds}
                    activeId={activeId}
                    cleanBusy={isCleaning}
                    allowReadonlyClean={allowReadonlyClean}
                    onFocus={setActiveId}
                    onToggleSelect={toggleSelected}
                    onCleanSingle={(id) => requestClean([id])}
                    onReveal={(path) => void handleReveal(path)}
                  />
                </Box>
              </Stack>
            </Paper>

            <InspectorPanel scanReport={scanReport} activeTarget={activeTarget} />
          </Box>
        </Stack>
      </Container>

      <Dialog
        open={Boolean(confirmTargetIds)}
        onClose={
          isCleaning
            ? undefined
            : () => {
                setConfirmTargetIds(null);
              }
        }
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>确认清理</DialogTitle>
        <DialogContent dividers>
          <Stack spacing={1}>
            <Typography variant="body2" sx={{ color: "text.secondary" }}>
              本次将处理 {confirmTargets.length} 个目标，预估释放 {formatBytes(confirmBytes)}。
            </Typography>
            {confirmTargets.map((item) => (
              <Stack
                key={item.id}
                direction="row"
                spacing={0.7}
                sx={{ justifyContent: "space-between", alignItems: "center" }}
              >
                <Typography variant="body2">{item.title}</Typography>
                <Typography variant="caption" sx={{ color: "text.secondary" }}>
                  {item.sizeLabel}
                </Typography>
              </Stack>
            ))}
            {confirmHasElevatedRisk ? (
              <>
                <Alert severity="warning">
                  当前选择含中高风险目标，确认后将直接清理目录内容。
                </Alert>
                <FormControlLabel
                  control={
                    <Checkbox
                      checked={elevatedRiskAcknowledged}
                      onChange={(event) => setElevatedRiskAcknowledged(event.target.checked)}
                    />
                  }
                  label="我已了解风险，继续清理"
                />
              </>
            ) : null}
            {confirmHasReadonly ? (
              <Alert severity="error">
                本次包含只读目标，将按专家模式直接执行清理，请确认当前选择无误。
              </Alert>
            ) : null}
          </Stack>
        </DialogContent>
        <DialogActions>
          <Button
            disabled={isCleaning}
            onClick={() => {
              setConfirmTargetIds(null);
            }}
          >
            取消
          </Button>
          <Button
            variant="contained"
            disabled={
              isCleaning || (confirmHasElevatedRisk && !elevatedRiskAcknowledged)
            }
            onClick={() => void handleConfirmClean()}
            startIcon={
              isCleaning ? <CircularProgress size={16} color="inherit" /> : undefined
            }
          >
            {isCleaning ? "清理中" : "确认清理"}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
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
    "疑似应用残留": "应用残留",
    "大文件概览": "大文件",
    "应用专属清理": "专属清理",
    "应用专属概览": "专属概览",
  };
  return map[label] ?? label;
}

export default App;
