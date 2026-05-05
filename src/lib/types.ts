export type RiskLevel = "low" | "medium" | "high";

export interface DiskSnapshot {
  rootPath: string;
  totalBytes: number;
  availableBytes: number;
  usedBytes: number;
  usedRatio: number;
  totalLabel: string;
  availableLabel: string;
  usedLabel: string;
}

export interface TargetScan {
  id: string;
  title: string;
  category: string;
  description: string;
  caution: string;
  risk: RiskLevel;
  defaultSelected: boolean;
  cleanable: boolean;
  expertCleanable: boolean;
  exists: boolean;
  path: string;
  relativePath: string;
  sizeBytes: number;
  sizeLabel: string;
  scanError: string | null;
}

export interface ScanReport {
  generatedAt: string;
  reclaimableBytes: number;
  reclaimableLabel: string;
  targetCount: number;
  existingCount: number;
  disk: DiskSnapshot;
  targets: TargetScan[];
}

export interface CleanedTarget {
  id: string;
  title: string;
  path: string;
  ok: boolean;
  dryRun: boolean;
  existedBefore: boolean;
  beforeBytes: number;
  afterBytes: number;
  freedBytes: number;
  beforeLabel: string;
  afterLabel: string;
  freedLabel: string;
  message: string;
}

export interface CleanReport {
  generatedAt: string;
  dryRun: boolean;
  targetCount: number;
  freedBytes: number;
  freedLabel: string;
  targets: CleanedTarget[];
}
