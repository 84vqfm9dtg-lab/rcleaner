use chrono::{SecondsFormat, Utc};
use glob::glob;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSnapshot {
    pub root_path: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub used_ratio: f64,
    pub total_label: String,
    pub available_label: String,
    pub used_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetScan {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub caution: String,
    pub risk: RiskLevel,
    pub default_selected: bool,
    pub cleanable: bool,
    pub expert_cleanable: bool,
    pub exists: bool,
    pub path: String,
    pub relative_path: String,
    pub size_bytes: u64,
    pub size_label: String,
    pub scan_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanReport {
    pub generated_at: String,
    pub reclaimable_bytes: u64,
    pub reclaimable_label: String,
    pub target_count: usize,
    pub existing_count: usize,
    pub disk: DiskSnapshot,
    pub targets: Vec<TargetScan>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanedTarget {
    pub id: String,
    pub title: String,
    pub path: String,
    pub ok: bool,
    pub dry_run: bool,
    pub existed_before: bool,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub freed_bytes: u64,
    pub before_label: String,
    pub after_label: String,
    pub freed_label: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanReport {
    pub generated_at: String,
    pub dry_run: bool,
    pub target_count: usize,
    pub freed_bytes: u64,
    pub freed_label: String,
    pub targets: Vec<CleanedTarget>,
}

#[derive(Debug, Clone)]
pub struct CleanOptions {
    pub target_ids: Vec<String>,
    pub clean_all: bool,
    pub dry_run: bool,
    pub allow_readonly: bool,
    pub root_override: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
struct TargetSpec {
    id: &'static str,
    title: &'static str,
    category: &'static str,
    description: &'static str,
    caution: &'static str,
    relative_path: &'static str,
    risk: RiskLevel,
    default_selected: bool,
    cleanable: bool,
}

#[derive(Debug, Clone, Copy)]
struct DynamicTargetFamily {
    id_prefix: &'static str,
    category: &'static str,
    description: &'static str,
    caution: &'static str,
    relative_root: &'static str,
    risk: RiskLevel,
    cleanable: bool,
    expert_cleanable: bool,
}

#[derive(Debug, Clone)]
struct RuntimeTargetSpec {
    id: String,
    title: String,
    category: String,
    description: String,
    caution: String,
    path: PathBuf,
    relative_path: String,
    risk: RiskLevel,
    default_selected: bool,
    cleanable: bool,
    expert_cleanable: bool,
}

#[derive(Debug, Clone)]
struct DynamicEntry {
    entry_name: String,
    relative_path: String,
    size_bytes: u64,
}

#[derive(Debug, Default)]
struct InstalledAppIndex {
    bundle_ids: HashSet<String>,
}

#[derive(Debug, Clone, Copy)]
struct AppProfileRule {
    id: &'static str,
    title: &'static str,
    category: &'static str,
    base_relative_path: &'static str,
    child_relative_path: &'static str,
    description: &'static str,
    caution: &'static str,
    risk: RiskLevel,
    cleanable: bool,
    expert_cleanable: bool,
}

#[derive(Debug, Deserialize)]
struct CustomRulesFile {
    version: Option<u32>,
    rules: Vec<CustomRuleSpec>,
}

#[derive(Debug, Deserialize)]
struct CustomRuleSpec {
    id: String,
    title: String,
    category: String,
    pattern: String,
    description: Option<String>,
    caution: Option<String>,
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default)]
    cleanable: bool,
    #[serde(default)]
    default_selected: bool,
    risk: Option<RiskLevel>,
}

const TARGET_SPECS: [TargetSpec; 13] = [
    TargetSpec {
        id: "system-logs",
        title: "系统日志",
        category: "系统维护",
        description: "清理当前用户目录下的日志文件，适合做轻量回收。",
        caution: "日志删除后无法回看旧记录，但通常不会影响应用运行。",
        relative_path: "Library/Logs",
        risk: RiskLevel::Low,
        default_selected: true,
        cleanable: true,
    },
    TargetSpec {
        id: "trash",
        title: "废纸篓",
        category: "系统维护",
        description: "清空当前用户废纸篓，是最直接的空间回收项。",
        caution: "废纸篓里的文件会被真正删除，请确认没有待恢复内容。",
        relative_path: ".Trash",
        risk: RiskLevel::Low,
        default_selected: true,
        cleanable: true,
    },
    TargetSpec {
        id: "npm-download-cache",
        title: "npm 下载缓存",
        category: "开发缓存",
        description: "清理 npm 的 tarball 与下载缓存，下次安装时会重新拉取。",
        caution: "不会移除已安装依赖，但后续 npm install 会重新下载缓存。",
        relative_path: ".npm/_cacache",
        risk: RiskLevel::Low,
        default_selected: true,
        cleanable: true,
    },
    TargetSpec {
        id: "pnpm-store",
        title: "pnpm Store",
        category: "开发缓存",
        description: "清理 pnpm 的全局包仓库，下次使用 pnpm 时会按需重建。",
        caution: "会影响后续 pnpm 安装速度，但不改项目源码。",
        relative_path: ".pnpm-store",
        risk: RiskLevel::Low,
        default_selected: false,
        cleanable: true,
    },
    TargetSpec {
        id: "cargo-registry-cache",
        title: "Cargo Registry Cache",
        category: "开发缓存",
        description: "清理 Cargo crate 下载缓存，下次构建会重新下载依赖包。",
        caution: "不会移除 toolchain，只会让后续 cargo build 重新下载部分包。",
        relative_path: ".cargo/registry/cache",
        risk: RiskLevel::Low,
        default_selected: true,
        cleanable: true,
    },
    TargetSpec {
        id: "cargo-git-cache",
        title: "Cargo Git Cache",
        category: "开发缓存",
        description: "清理 Cargo git 依赖缓存，适合整理长期积累的仓库拉取数据。",
        caution: "使用 git 依赖的 Rust 项目下次构建会重新拉取。",
        relative_path: ".cargo/git",
        risk: RiskLevel::Low,
        default_selected: false,
        cleanable: true,
    },
    TargetSpec {
        id: "playwright-cache",
        title: "Playwright 浏览器缓存",
        category: "自动化工具",
        description: "清理 Playwright 下载的浏览器运行时，适合自动化调试后回收空间。",
        caution: "后续自动化或测试首次运行时会重新下载浏览器。",
        relative_path: "Library/Caches/ms-playwright",
        risk: RiskLevel::Low,
        default_selected: true,
        cleanable: true,
    },
    TargetSpec {
        id: "homebrew-cache",
        title: "Homebrew 缓存",
        category: "开发缓存",
        description: "清理 Homebrew 下载缓存，适合安装过大量 formula 之后做整理。",
        caution: "只会影响后续 brew install 的缓存命中，不会卸载已装软件。",
        relative_path: "Library/Caches/Homebrew",
        risk: RiskLevel::Low,
        default_selected: false,
        cleanable: true,
    },
    TargetSpec {
        id: "pip-cache",
        title: "pip 缓存",
        category: "开发缓存",
        description: "清理 pip wheel 与下载缓存，适合 Python 环境较多的机器。",
        caution: "后续 pip install 会重新下载需要的包。",
        relative_path: "Library/Caches/pip",
        risk: RiskLevel::Low,
        default_selected: false,
        cleanable: true,
    },
    TargetSpec {
        id: "xcode-derived-data",
        title: "Xcode DerivedData",
        category: "Apple 开发",
        description: "清理 Xcode 派生构建产物，通常能回收较大空间。",
        caution: "下次打开对应工程时会重新索引和编译，适合空闲时清理。",
        relative_path: "Library/Developer/Xcode/DerivedData",
        risk: RiskLevel::Medium,
        default_selected: false,
        cleanable: true,
    },
    TargetSpec {
        id: "codex-home",
        title: "Codex 用户目录",
        category: "AI 开发工具（只读）",
        description: "扫描 ~/.codex 的整体占用，用于定位 Codex 会话与配置数据体积。",
        caution: "该目录可能包含技能、会话与本地配置，默认只读不提供直接清理。",
        relative_path: ".codex",
        risk: RiskLevel::Medium,
        default_selected: false,
        cleanable: false,
    },
    TargetSpec {
        id: "cursor-home",
        title: "Cursor 用户目录",
        category: "AI 开发工具（只读）",
        description: "扫描 ~/.cursor 的整体占用，帮助判断 Cursor 本地数据是否偏大。",
        caution: "该目录可能包含扩展与工作区元数据，默认只读不提供直接清理。",
        relative_path: ".cursor",
        risk: RiskLevel::Medium,
        default_selected: false,
        cleanable: false,
    },
    TargetSpec {
        id: "kiro-home",
        title: "Kiro 用户目录",
        category: "AI 开发工具（只读）",
        description: "扫描 ~/.kiro 的整体占用，用于识别 Kiro 相关本地数据体积。",
        caution: "该目录可能包含项目与会话数据，默认只读不提供直接清理。",
        relative_path: ".kiro",
        risk: RiskLevel::Medium,
        default_selected: false,
        cleanable: false,
    },
];

const DYNAMIC_TARGET_FAMILIES: [DynamicTargetFamily; 3] = [
    DynamicTargetFamily {
        id_prefix: "app-cache",
        category: "应用缓存（可清理）",
        description: "应用缓存目录（按应用分组），通常可以安全回收，后续应用会按需重建。",
        caution: "清理后首次打开应用可能会变慢，少数应用需要重新下载缓存内容。",
        relative_root: "Library/Caches",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    DynamicTargetFamily {
        id_prefix: "app-support",
        category: "应用支持数据（只读）",
        description: "应用支持目录（按应用分组），用于识别系统数据大户，默认仅展示不直接清理。",
        caution: "这里可能包含业务数据或离线内容，误删会导致应用数据丢失，默认只读。",
        relative_root: "Library/Application Support",
        risk: RiskLevel::Medium,
        cleanable: false,
        expert_cleanable: false,
    },
    DynamicTargetFamily {
        id_prefix: "app-container",
        category: "应用容器（只读）",
        description: "沙盒容器目录（按应用分组），常见于 macOS 应用系统数据，默认只读。",
        caution: "容器目录删除风险高，可能导致应用配置或文档丢失，默认只读。",
        relative_root: "Library/Containers",
        risk: RiskLevel::High,
        cleanable: false,
        expert_cleanable: false,
    },
];

const DYNAMIC_SCAN_MIN_BYTES: u64 = 128 * 1024 * 1024;
const DYNAMIC_SCAN_MAX_PER_FAMILY: usize = 24;
const LARGE_FILE_MIN_BYTES: u64 = 512 * 1024 * 1024;
const LARGE_FILE_MAX_COUNT: usize = 24;

const APP_PROFILE_RULES: [AppProfileRule; 8] = [
    AppProfileRule {
        id: "docker-logs",
        title: "Docker Desktop · Logs",
        category: "应用专属清理",
        base_relative_path: "Library/Containers/com.docker.docker",
        child_relative_path: "Data/log",
        description: "Docker Desktop 运行日志。",
        caution: "仅清理日志文件，不删除镜像、容器或卷。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "docker-vms",
        title: "Docker Desktop · VM Data",
        category: "应用专属概览",
        base_relative_path: "Library/Containers/com.docker.docker",
        child_relative_path: "Data/vms",
        description: "Docker Desktop 虚拟机数据，通常包含镜像、容器和卷。",
        caution: "这是 Docker 核心数据，只做概览。需要通过 Docker 自身命令清理。",
        risk: RiskLevel::High,
        cleanable: false,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "netease-caches",
        title: "网易云音乐 · Caches",
        category: "应用专属清理",
        base_relative_path: "Library/Containers/com.netease.163music",
        child_relative_path: "Data/Caches",
        description: "网易云音乐容器缓存。",
        caution: "仅清理缓存目录，离线音乐和用户文档不在此项内。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "netease-tmp",
        title: "网易云音乐 · tmp",
        category: "应用专属清理",
        base_relative_path: "Library/Containers/com.netease.163music",
        child_relative_path: "Data/tmp",
        description: "网易云音乐临时文件。",
        caution: "仅清理临时文件。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "wps-caches",
        title: "WPS Office · Caches",
        category: "应用专属清理",
        base_relative_path: "Library/Containers/com.kingsoft.wpsoffice.mac",
        child_relative_path: "Data/Caches",
        description: "WPS Office 容器缓存。",
        caution: "仅清理缓存目录，不删除文档。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "wps-tmp",
        title: "WPS Office · tmp",
        category: "应用专属清理",
        base_relative_path: "Library/Containers/com.kingsoft.wpsoffice.mac",
        child_relative_path: "Data/tmp",
        description: "WPS Office 临时文件。",
        caution: "仅清理临时文件。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "cursor-cache",
        title: "Cursor · Cache",
        category: "应用专属清理",
        base_relative_path: "Library/Application Support/Cursor",
        child_relative_path: "Cache",
        description: "Cursor 应用缓存。",
        caution: "仅清理缓存目录，不删除工作区配置。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
    AppProfileRule {
        id: "cursor-cached-data",
        title: "Cursor · CachedData",
        category: "应用专属清理",
        base_relative_path: "Library/Application Support/Cursor",
        child_relative_path: "CachedData",
        description: "Cursor 运行时缓存数据。",
        caution: "清理后 Cursor 可能需要重新生成部分缓存。",
        risk: RiskLevel::Low,
        cleanable: true,
        expert_cleanable: false,
    },
];

pub fn app_name() -> &'static str {
    "rCleaner"
}

pub fn binary_name() -> &'static str {
    "rcleaner"
}

pub fn app_identifier() -> &'static str {
    "app.rseries.rcleaner"
}

pub fn scan_targets(root_override: Option<PathBuf>) -> Result<ScanReport, String> {
    let root = resolve_root(root_override)?;
    let disk = read_disk_snapshot(&root)?;
    let specs = collect_runtime_specs(&root)?;
    let mut targets = Vec::with_capacity(specs.len());
    let mut reclaimable_bytes = 0_u64;
    let mut existing_count = 0_usize;

    for spec in specs {
        let path = spec.path.clone();
        let path_exists = path.exists();
        if path_exists {
            existing_count += 1;
        }

        let measurement = measure_path(&path);
        let size_bytes = measurement.as_ref().copied().unwrap_or(0);
        if spec.cleanable {
            reclaimable_bytes = reclaimable_bytes.saturating_add(size_bytes);
        }

        targets.push(TargetScan {
            id: spec.id,
            title: spec.title,
            category: spec.category,
            description: spec.description,
            caution: spec.caution,
            risk: spec.risk,
            default_selected: spec.default_selected,
            cleanable: spec.cleanable,
            expert_cleanable: spec.expert_cleanable,
            exists: path_exists,
            path: absolute_path(&path).to_string_lossy().to_string(),
            relative_path: spec.relative_path,
            size_bytes,
            size_label: format_bytes(size_bytes),
            scan_error: measurement.err(),
        });
    }

    targets.sort_by(|left, right| {
        right
            .size_bytes
            .cmp(&left.size_bytes)
            .then_with(|| left.title.cmp(&right.title))
    });

    Ok(ScanReport {
        generated_at: now_iso(),
        reclaimable_bytes,
        reclaimable_label: format_bytes(reclaimable_bytes),
        target_count: targets.len(),
        existing_count,
        disk,
        targets,
    })
}

pub fn clean_targets(options: CleanOptions) -> Result<CleanReport, String> {
    if options.clean_all && !options.target_ids.is_empty() {
        return Err("use either --all or --target, not both".to_string());
    }

    let root = resolve_root(options.root_override)?;
    let runtime_specs = collect_runtime_specs(&root)?;

    let selected_specs = if options.clean_all {
        runtime_specs
            .iter()
            .filter(|item| item.cleanable || (options.allow_readonly && item.expert_cleanable))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        if options.target_ids.is_empty() {
            return Err("at least one --target is required unless --all is used".to_string());
        }

        let mut resolved = Vec::with_capacity(options.target_ids.len());
        for target_id in &options.target_ids {
            let spec = runtime_specs
                .iter()
                .find(|item| item.id == *target_id)
                .ok_or_else(|| format!("unknown target id: {target_id}"))?;
            if !spec.cleanable && !(options.allow_readonly && spec.expert_cleanable) {
                return Err(format!(
                    "target id is read-only and cannot be cleaned directly: {target_id}"
                ));
            }
            resolved.push(spec.clone());
        }
        resolved
    };

    let mut cleaned_targets = Vec::with_capacity(selected_specs.len());
    let mut freed_bytes = 0_u64;

    for spec in selected_specs {
        let path = spec.path.clone();
        let existed_before = path.exists();
        let before_bytes = measure_path(&path).unwrap_or(0);

        let (after_bytes, message, ok) = if !existed_before {
            (0_u64, "target path does not exist".to_string(), true)
        } else if options.dry_run {
            (
                before_bytes,
                "dry run only, nothing was removed".to_string(),
                true,
            )
        } else {
            match clear_path_contents(&path) {
                Ok(()) => {
                    let after = measure_path(&path).unwrap_or(0);
                    (after, "cleaned target contents".to_string(), true)
                }
                Err(error) => (
                    before_bytes,
                    format!("failed to clean target: {error}"),
                    false,
                ),
            }
        };

        let item_freed = before_bytes.saturating_sub(after_bytes);
        freed_bytes = freed_bytes.saturating_add(item_freed);

        cleaned_targets.push(CleanedTarget {
            id: spec.id,
            title: spec.title,
            path: absolute_path(&path).to_string_lossy().to_string(),
            ok,
            dry_run: options.dry_run,
            existed_before,
            before_bytes,
            after_bytes,
            freed_bytes: item_freed,
            before_label: format_bytes(before_bytes),
            after_label: format_bytes(after_bytes),
            freed_label: format_bytes(item_freed),
            message,
        });
    }

    Ok(CleanReport {
        generated_at: now_iso(),
        dry_run: options.dry_run,
        target_count: cleaned_targets.len(),
        freed_bytes,
        freed_label: format_bytes(freed_bytes),
        targets: cleaned_targets,
    })
}

pub fn reveal_in_finder(path: PathBuf) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg("-R")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            return Ok(());
        }
        return Err("failed to reveal path in Finder".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("explorer")
            .arg(path)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            return Ok(());
        }
        return Err("failed to reveal path in Explorer".to_string());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = if path.is_dir() {
            path
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| PathBuf::from("/"))
        };
        let status = Command::new("xdg-open")
            .arg(target)
            .status()
            .map_err(|error| error.to_string())?;
        if status.success() {
            return Ok(());
        }
        return Err("failed to reveal path".to_string());
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit_index = 0_usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

fn collect_runtime_specs(root: &Path) -> Result<Vec<RuntimeTargetSpec>, String> {
    let app_index = build_installed_app_index(root);
    let mut specs = TARGET_SPECS
        .iter()
        .map(|spec| RuntimeTargetSpec {
            id: spec.id.to_string(),
            title: spec.title.to_string(),
            category: spec.category.to_string(),
            description: spec.description.to_string(),
            caution: spec.caution.to_string(),
            path: root.join(spec.relative_path),
            relative_path: spec.relative_path.to_string(),
            risk: spec.risk,
            default_selected: spec.default_selected,
            cleanable: spec.cleanable,
            expert_cleanable: false,
        })
        .collect::<Vec<_>>();

    specs.extend(discover_dynamic_specs(root, &app_index));
    specs.extend(discover_container_cache_specs(root));
    specs.extend(discover_app_profile_specs(root));
    specs.extend(discover_large_file_specs(root));
    specs.extend(discover_custom_specs(root)?);
    let mut deduped = dedupe_runtime_specs(specs);
    protect_active_runtime_specs(&mut deduped);
    Ok(deduped)
}

fn protect_active_runtime_specs(specs: &mut [RuntimeTargetSpec]) {
    let workspace_root = std::env::current_dir()
        .ok()
        .map(|path| absolute_path(&path));
    let current_executable = std::env::current_exe()
        .ok()
        .map(|path| absolute_path(&path));

    for spec in specs.iter_mut() {
        if !spec.cleanable {
            continue;
        }

        if should_lock_runtime_target(
            &spec.path,
            workspace_root.as_deref(),
            current_executable.as_deref(),
        ) {
            spec.cleanable = false;
            spec.expert_cleanable = false;
            spec.default_selected = false;
            spec.caution = format!(
                "{} 当前 rCleaner 正在使用该目录，已临时锁定，建议退出应用后再处理。",
                spec.caution
            );
        }
    }
}

fn should_lock_runtime_target(
    path: &Path,
    workspace_root: Option<&Path>,
    current_executable: Option<&Path>,
) -> bool {
    let path = absolute_path(path);

    if let Some(executable) = current_executable {
        if executable.starts_with(&path) {
            return true;
        }
    }

    if let Some(workspace_root) = workspace_root {
        if path.starts_with(workspace_root) {
            let terminal_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("");
            if matches!(terminal_name, "target" | "dist" | "node_modules" | ".vite") {
                return true;
            }
        }
    }

    false
}

fn build_installed_app_index(root: &Path) -> InstalledAppIndex {
    let mut index = InstalledAppIndex::default();
    let mut search_roots = vec![root.join("Applications")];

    if let Some(home) = dirs::home_dir() {
        search_roots.push(home.join("Applications"));
    }
    search_roots.extend([
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
    ]);

    let mut visited_roots = HashSet::new();
    for search_root in search_roots {
        let search_root = absolute_path(&search_root);
        if !visited_roots.insert(search_root.clone()) || !search_root.exists() {
            continue;
        }

        for entry in WalkDir::new(search_root)
            .max_depth(3)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if !is_app_bundle_path(path) {
                continue;
            }
            if let Some(bundle_id) = read_bundle_identifier(path) {
                index.bundle_ids.insert(bundle_id);
            }
        }
    }

    index
}

fn is_app_bundle_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
}

fn read_bundle_identifier(app_path: &Path) -> Option<String> {
    let info_path = app_path.join("Contents/Info.plist");
    let plist = plist::Value::from_file(info_path).ok()?;
    plist
        .as_dictionary()?
        .get("CFBundleIdentifier")?
        .as_string()
        .map(str::to_string)
}

fn discover_dynamic_specs(root: &Path, app_index: &InstalledAppIndex) -> Vec<RuntimeTargetSpec> {
    let mut specs = Vec::new();
    for family in DYNAMIC_TARGET_FAMILIES {
        let entries = discover_dynamic_entries(root, family);
        for entry in entries {
            let orphan_kind = orphan_kind_for_dynamic_entry(family, &entry.entry_name, app_index);
            specs.push(RuntimeTargetSpec {
                id: format!("{}:{}", family.id_prefix, entry.entry_name),
                title: entry.entry_name,
                category: orphan_kind
                    .as_ref()
                    .map(|_| "疑似应用残留".to_string())
                    .unwrap_or_else(|| family.category.to_string()),
                description: orphan_kind
                    .as_ref()
                    .map(|kind| {
                        format!("未在常见应用目录中找到对应 App，可能是卸载后保留的{kind}。")
                    })
                    .unwrap_or_else(|| family.description.to_string()),
                caution: orphan_kind
                    .as_ref()
                    .map(|_| "建议先打开目录确认内容；专家模式下可清理该残留候选。".to_string())
                    .unwrap_or_else(|| family.caution.to_string()),
                path: root.join(&entry.relative_path),
                relative_path: entry.relative_path,
                risk: orphan_kind.map_or(family.risk, |_| RiskLevel::High),
                default_selected: false,
                cleanable: family.cleanable && orphan_kind.is_none(),
                expert_cleanable: orphan_kind.is_some() || family.expert_cleanable,
            });
        }
    }
    specs
}

fn orphan_kind_for_dynamic_entry(
    family: DynamicTargetFamily,
    entry_name: &str,
    app_index: &InstalledAppIndex,
) -> Option<&'static str> {
    if !matches!(family.id_prefix, "app-support" | "app-container") {
        return None;
    }
    if !looks_like_bundle_identifier(entry_name) || is_system_bundle_identifier(entry_name) {
        return None;
    }
    if app_index.bundle_ids.contains(entry_name) {
        return None;
    }

    match family.id_prefix {
        "app-support" => Some("应用支持数据"),
        "app-container" => Some("应用容器"),
        _ => None,
    }
}

fn looks_like_bundle_identifier(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() >= 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
        })
}

fn is_system_bundle_identifier(value: &str) -> bool {
    value.starts_with("com.apple.")
        || value.starts_with("com.microsoft.")
        || value.starts_with("com.google.")
}

fn discover_container_cache_specs(root: &Path) -> Vec<RuntimeTargetSpec> {
    const CONTAINER_CACHE_PATHS: [(&str, &str, RiskLevel); 3] = [
        ("Data/Caches", "Caches", RiskLevel::Low),
        ("Data/tmp", "tmp", RiskLevel::Low),
        ("Data/Library/Caches", "Library Caches", RiskLevel::Low),
    ];

    let base_path = root.join("Library/Containers");
    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut specs = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let app_name = entry.file_name().to_string_lossy().to_string();
        if app_name.is_empty() {
            continue;
        }

        for (child_relative, child_label, risk) in CONTAINER_CACHE_PATHS {
            let relative_path = Path::new("Library/Containers")
                .join(&app_name)
                .join(child_relative);
            let absolute_path = root.join(&relative_path);
            let size_bytes = measure_path(&absolute_path).unwrap_or(0);
            if size_bytes == 0 {
                continue;
            }

            specs.push(RuntimeTargetSpec {
                id: format!(
                    "app-container-cache:{}:{}",
                    app_name,
                    slug_path_fragment(child_relative)
                ),
                title: format!("{app_name} · {child_label}"),
                category: "应用容器缓存".to_string(),
                description: "来自应用容器内部的缓存/临时目录，适合精细清理。".to_string(),
                caution: "仅清理容器中的缓存子目录，不会直接删除整个应用容器。".to_string(),
                path: absolute_path,
                relative_path: relative_path.to_string_lossy().to_string(),
                risk,
                default_selected: false,
                cleanable: true,
                expert_cleanable: false,
            });
        }
    }

    specs
}

fn discover_app_profile_specs(root: &Path) -> Vec<RuntimeTargetSpec> {
    let mut specs = Vec::new();
    for rule in APP_PROFILE_RULES {
        let relative_path = Path::new(rule.base_relative_path).join(rule.child_relative_path);
        let path = root.join(&relative_path);
        let size_bytes = measure_path(&path).unwrap_or(0);
        if size_bytes == 0 {
            continue;
        }

        specs.push(RuntimeTargetSpec {
            id: format!("profile:{}", rule.id),
            title: rule.title.to_string(),
            category: rule.category.to_string(),
            description: rule.description.to_string(),
            caution: rule.caution.to_string(),
            path,
            relative_path: relative_path.to_string_lossy().to_string(),
            risk: rule.risk,
            default_selected: false,
            cleanable: rule.cleanable,
            expert_cleanable: rule.expert_cleanable,
        });
    }

    specs
}

fn discover_large_file_specs(root: &Path) -> Vec<RuntimeTargetSpec> {
    let scan_roots = ["Downloads", "Desktop", "Documents"];
    let mut entries = Vec::new();

    for relative_root in scan_roots {
        let base_path = root.join(relative_root);
        if !base_path.exists() {
            continue;
        }

        for entry in WalkDir::new(&base_path)
            .max_depth(4)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            let path = entry.path();
            if should_skip_large_file_path(path) {
                continue;
            }

            let metadata = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if !metadata.is_file() || metadata.len() < LARGE_FILE_MIN_BYTES {
                continue;
            }

            let size_bytes = measure_file_disk_usage(&metadata).max(metadata.len());
            entries.push((path.to_path_buf(), size_bytes));
        }
    }

    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries
        .into_iter()
        .take(LARGE_FILE_MAX_COUNT)
        .map(|(path, _size_bytes)| {
            let relative_path = display_relative_path(root, &path);
            let title = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("large file")
                .to_string();
            RuntimeTargetSpec {
                id: format!("large-file:{}", slug_path_fragment(&relative_path)),
                title,
                category: "大文件概览".to_string(),
                description: "用户目录中的大文件，用于定位空间占用，不会直接清理。".to_string(),
                caution: "这是用户文件，请打开确认后手动处理。".to_string(),
                path,
                relative_path,
                risk: RiskLevel::Medium,
                default_selected: false,
                cleanable: false,
                expert_cleanable: false,
            }
        })
        .collect()
}

fn should_skip_large_file_path(path: &Path) -> bool {
    path.components().any(|component| {
        let name = component.as_os_str().to_string_lossy();
        matches!(
            name.as_ref(),
            "node_modules" | "target" | ".git" | ".Trash" | "Library"
        )
    })
}

fn discover_dynamic_entries(root: &Path, family: DynamicTargetFamily) -> Vec<DynamicEntry> {
    let base_path = root.join(family.relative_root);
    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut discovered = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let entry_name = entry.file_name().to_string_lossy().to_string();
        if entry_name.is_empty() {
            continue;
        }

        let relative_path = Path::new(family.relative_root)
            .join(entry.file_name())
            .to_string_lossy()
            .to_string();
        let target_path = root.join(&relative_path);
        let size_bytes = measure_path(&target_path).unwrap_or(0);
        if size_bytes == 0 {
            continue;
        }

        discovered.push(DynamicEntry {
            entry_name,
            relative_path,
            size_bytes,
        });
    }

    discovered.sort_by(|left, right| {
        right
            .size_bytes
            .cmp(&left.size_bytes)
            .then_with(|| left.entry_name.cmp(&right.entry_name))
    });

    let has_large_entry = discovered
        .iter()
        .any(|entry| entry.size_bytes >= DYNAMIC_SCAN_MIN_BYTES);

    discovered
        .into_iter()
        .filter(|entry| !has_large_entry || entry.size_bytes >= DYNAMIC_SCAN_MIN_BYTES)
        .take(DYNAMIC_SCAN_MAX_PER_FAMILY)
        .collect()
}

fn discover_custom_specs(root: &Path) -> Result<Vec<RuntimeTargetSpec>, String> {
    let mut specs = Vec::new();
    for config_path in custom_rule_config_paths(root) {
        if !config_path.exists() {
            continue;
        }

        let contents = fs::read_to_string(&config_path).map_err(|error| {
            format!(
                "failed to read custom rules file {}: {error}",
                config_path.to_string_lossy()
            )
        })?;
        let parsed: CustomRulesFile = toml::from_str(&contents).map_err(|error| {
            format!(
                "failed to parse custom rules file {}: {error}",
                config_path.to_string_lossy()
            )
        })?;

        if let Some(version) = parsed.version {
            if version != 1 {
                return Err(format!(
                    "unsupported custom rules version in {}: {}",
                    config_path.to_string_lossy(),
                    version
                ));
            }
        }

        for rule in parsed.rules.into_iter().filter(|rule| rule.enabled) {
            specs.extend(resolve_custom_rule(root, &config_path, rule)?);
        }
    }

    Ok(specs)
}

fn resolve_custom_rule(
    root: &Path,
    config_path: &Path,
    rule: CustomRuleSpec,
) -> Result<Vec<RuntimeTargetSpec>, String> {
    let home = root;
    let expanded_pattern = expand_rule_pattern(home, config_path, &rule.pattern);
    let mut matches = Vec::new();

    let entries = glob(&expanded_pattern)
        .map_err(|error| format!("invalid glob pattern {}: {}", rule.pattern, error))?;
    for entry in entries {
        let path = match entry {
            Ok(path) => path,
            Err(error) => {
                return Err(format!(
                    "failed while matching custom rule {}: {}",
                    rule.id, error
                ))
            }
        };

        if !path.exists() || !path.starts_with(home) {
            continue;
        }

        let title = build_custom_title(&rule.title, &path);
        let relative_path = display_relative_path(home, &path);
        let path_slug = slug_path_fragment(&relative_path);
        matches.push(RuntimeTargetSpec {
            id: format!("custom:{}:{}", rule.id, path_slug),
            title,
            category: rule.category.clone(),
            description: rule
                .description
                .clone()
                .unwrap_or_else(|| format!("来自自定义规则：{}", rule.title)),
            caution: rule.caution.clone().unwrap_or_else(|| {
                if rule.cleanable {
                    "来自自定义规则，清理前请确认该路径确实属于可回收缓存或构建产物。".to_string()
                } else {
                    "来自自定义规则，当前按只读展示。".to_string()
                }
            }),
            path: path.clone(),
            relative_path,
            risk: rule.risk.unwrap_or(if rule.cleanable {
                RiskLevel::Medium
            } else {
                RiskLevel::High
            }),
            default_selected: rule.default_selected,
            cleanable: rule.cleanable,
            expert_cleanable: false,
        });
    }

    Ok(matches)
}

fn custom_rule_config_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.join(".rcleaner/rules.toml")];
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join("rules/cleaner-rules.toml"));
    }
    paths
}

fn expand_rule_pattern(home: &Path, config_path: &Path, pattern: &str) -> String {
    if let Some(stripped) = pattern.strip_prefix("~/") {
        return home.join(stripped).to_string_lossy().to_string();
    }
    let candidate = Path::new(pattern);
    if candidate.is_absolute() {
        return candidate.to_string_lossy().to_string();
    }
    config_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(candidate)
        .to_string_lossy()
        .to_string()
}

fn build_custom_title(title: &str, path: &Path) -> String {
    let context = infer_custom_context_name(path);
    if context.is_empty() {
        title.to_string()
    } else {
        format!("{context} · {title}")
    }
}

fn infer_custom_context_name(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let parent_name = path
        .parent()
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .unwrap_or("");

    if matches!(
        file_name,
        "target" | "dist" | "node_modules" | ".vite" | "coverage"
    ) {
        if parent_name == "src-tauri" {
            return path
                .parent()
                .and_then(|value| value.parent())
                .and_then(|value| value.file_name())
                .and_then(|value| value.to_str())
                .unwrap_or("")
                .to_string();
        }
        return parent_name.to_string();
    }

    parent_name.to_string()
}

fn display_relative_path(home: &Path, path: &Path) -> String {
    path.strip_prefix(home)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

fn slug_path_fragment(path: &str) -> String {
    path.chars()
        .map(|char| match char {
            'a'..='z' | 'A'..='Z' | '0'..='9' => char,
            _ => '-',
        })
        .collect()
}

fn dedupe_runtime_specs(specs: Vec<RuntimeTargetSpec>) -> Vec<RuntimeTargetSpec> {
    let mut seen = HashSet::new();
    let mut deduped = Vec::with_capacity(specs.len());
    for spec in specs {
        if seen.insert(spec.id.clone()) {
            deduped.push(spec);
        }
    }
    deduped
}

fn resolve_root(root_override: Option<PathBuf>) -> Result<PathBuf, String> {
    let candidate = if let Some(root) = root_override {
        root
    } else {
        dirs::home_dir()
            .ok_or_else(|| "failed to resolve the current home directory".to_string())?
    };

    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(candidate))
            .map_err(|error| error.to_string())
    }
}

fn read_disk_snapshot(root: &Path) -> Result<DiskSnapshot, String> {
    let total_bytes = fs2::total_space(root).map_err(|error| error.to_string())?;
    let available_bytes = fs2::available_space(root).map_err(|error| error.to_string())?;
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let used_ratio = if total_bytes == 0 {
        0.0
    } else {
        used_bytes as f64 / total_bytes as f64
    };

    Ok(DiskSnapshot {
        root_path: absolute_path(root).to_string_lossy().to_string(),
        total_bytes,
        available_bytes,
        used_bytes,
        used_ratio,
        total_label: format_bytes(total_bytes),
        available_label: format_bytes(available_bytes),
        used_label: format_bytes(used_bytes),
    })
}

fn measure_path(path: &Path) -> Result<u64, String> {
    if !path.exists() {
        return Ok(0);
    }

    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.is_file() || metadata.file_type().is_symlink() {
        return Ok(measure_file_disk_usage(&metadata));
    }

    let mut total = 0_u64;
    for entry in WalkDir::new(path).follow_links(false) {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        if metadata.is_file() || metadata.file_type().is_symlink() {
            total = total.saturating_add(measure_file_disk_usage(&metadata));
        }
    }

    Ok(total)
}

fn measure_file_disk_usage(metadata: &fs::Metadata) -> u64 {
    #[cfg(unix)]
    {
        // st_blocks is measured in 512-byte units and better reflects real disk usage
        // for sparse files (for example Docker.raw) than metadata.len().
        return metadata.blocks().saturating_mul(512);
    }

    #[cfg(not(unix))]
    {
        metadata.len()
    }
}

fn default_true() -> bool {
    true
}

fn clear_path_contents(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;

    if metadata.is_file() || metadata.file_type().is_symlink() {
        fs::remove_file(path).map_err(|error| error.to_string())?;
        return Ok(());
    }

    let entries = fs::read_dir(path).map_err(|error| error.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        remove_entry(entry.path())?;
    }

    Ok(())
}

fn remove_entry(path: PathBuf) -> Result<(), String> {
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).map_err(|error| error.to_string())
    } else {
        fs::remove_file(path).map_err(|error| error.to_string())
    }
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(path)
}

fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn target_ids() -> Vec<OsString> {
    TARGET_SPECS
        .iter()
        .map(|target| OsString::from(target.id))
        .collect()
}
