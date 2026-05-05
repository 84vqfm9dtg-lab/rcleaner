import { invoke } from "@tauri-apps/api/core";
import { createMockCleanReport, createMockScanReport } from "./mock-data";
import type { CleanReport, ScanReport } from "./types";

function isTauriRuntime(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

export async function scanTargets(): Promise<ScanReport> {
  if (!isTauriRuntime()) {
    return createMockScanReport();
  }
  return invoke<ScanReport>("scan_targets_command");
}

export async function cleanTargets(
  targetIds: string[],
  dryRun = false,
  allowReadonly = false,
): Promise<CleanReport> {
  if (!isTauriRuntime()) {
    return createMockCleanReport(targetIds);
  }
  return invoke<CleanReport>("clean_targets_command", { targetIds, dryRun, allowReadonly });
}

export async function revealPath(path: string): Promise<void> {
  if (!isTauriRuntime()) {
    return;
  }
  await invoke("reveal_path_command", { path });
}
