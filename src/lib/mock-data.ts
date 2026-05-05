import type { CleanReport, ScanReport } from "./types";

export function createMockScanReport(): ScanReport {
  const generatedAt = new Date().toISOString();
  return {
    generatedAt,
    reclaimableBytes: 4_960_000_000,
    reclaimableLabel: "4.6 GB",
    targetCount: 5,
    existingCount: 5,
    disk: {
      rootPath: "~/Demo",
      totalBytes: 494_000_000_000,
      availableBytes: 35_000_000_000,
      usedBytes: 459_000_000_000,
      usedRatio: 0.93,
      totalLabel: "460.1 GB",
      availableLabel: "32.6 GB",
      usedLabel: "427.5 GB",
    },
    targets: [
      {
        id: "playwright-cache",
        title: "Playwright 浏览器缓存",
        category: "自动化工具",
        description: "清理 Playwright 下载的浏览器运行时，适合自动化调试后回收空间。",
        caution: "后续自动化或测试首次运行时会重新下载浏览器。",
        risk: "low",
        defaultSelected: true,
        cleanable: true,
        expertCleanable: false,
        exists: true,
        path: "~/Demo/Library/Caches/ms-playwright",
        relativePath: "Library/Caches/ms-playwright",
        sizeBytes: 1_780_000_000,
        sizeLabel: "1.7 GB",
        scanError: null,
      },
      {
        id: "homebrew-cache",
        title: "Homebrew 缓存",
        category: "开发缓存",
        description: "清理 Homebrew 下载缓存，适合安装过大量 formula 之后做整理。",
        caution: "只会影响后续 brew install 的缓存命中，不会卸载已装软件。",
        risk: "low",
        defaultSelected: false,
        cleanable: true,
        expertCleanable: false,
        exists: true,
        path: "~/Demo/Library/Caches/Homebrew",
        relativePath: "Library/Caches/Homebrew",
        sizeBytes: 2_180_000_000,
        sizeLabel: "2.0 GB",
        scanError: null,
      },
      {
        id: "npm-download-cache",
        title: "npm 下载缓存",
        category: "开发缓存",
        description: "清理 npm 的 tarball 与下载缓存，下次安装时会重新拉取。",
        caution: "不会移除已安装依赖，但后续 npm install 会重新下载缓存。",
        risk: "low",
        defaultSelected: true,
        cleanable: true,
        expertCleanable: false,
        exists: true,
        path: "~/Demo/.npm/_cacache",
        relativePath: ".npm/_cacache",
        sizeBytes: 820_000_000,
        sizeLabel: "782.0 MB",
        scanError: null,
      },
      {
        id: "system-logs",
        title: "系统日志",
        category: "系统维护",
        description: "清理当前用户目录下的日志文件，适合做轻量回收。",
        caution: "日志删除后无法回看旧记录，但通常不会影响应用运行。",
        risk: "low",
        defaultSelected: true,
        cleanable: true,
        expertCleanable: false,
        exists: true,
        path: "~/Demo/Library/Logs",
        relativePath: "Library/Logs",
        sizeBytes: 160_000_000,
        sizeLabel: "152.6 MB",
        scanError: null,
      },
      {
        id: "xcode-derived-data",
        title: "Xcode DerivedData",
        category: "Apple 开发",
        description: "清理 Xcode 派生构建产物，通常能回收较大空间。",
        caution: "下次打开对应工程时会重新索引和编译，适合空闲时清理。",
        risk: "medium",
        defaultSelected: false,
        cleanable: true,
        expertCleanable: false,
        exists: true,
        path: "~/Demo/Library/Developer/Xcode/DerivedData",
        relativePath: "Library/Developer/Xcode/DerivedData",
        sizeBytes: 120_000_000,
        sizeLabel: "114.4 MB",
        scanError: null,
      },
    ],
  };
}

export function createMockCleanReport(targetIds: string[]): CleanReport {
  const scan = createMockScanReport();
  const targets = scan.targets.filter((item) => targetIds.includes(item.id));
  const freedBytes = targets.reduce((sum, item) => sum + item.sizeBytes, 0);
  return {
    generatedAt: new Date().toISOString(),
    dryRun: false,
    targetCount: targets.length,
    freedBytes,
    freedLabel: formatBytes(freedBytes),
    targets: targets.map((item) => ({
      id: item.id,
      title: item.title,
      path: item.path,
      ok: true,
      dryRun: false,
      existedBefore: true,
      beforeBytes: item.sizeBytes,
      afterBytes: 0,
      freedBytes: item.sizeBytes,
      beforeLabel: item.sizeLabel,
      afterLabel: "0 B",
      freedLabel: item.sizeLabel,
      message: "mock cleanup completed",
    })),
  };
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
