use serde_json::Value;
use std::process::{Command, Output};

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_rcleaner")
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(binary_path())
        .args(args)
        .output()
        .expect("failed to run rcleaner binary")
}

fn parse_stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should contain valid json")
}

fn write_fixture_file(root: &std::path::Path, relative_path: &str, size: usize) {
    let path = root.join(relative_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("failed to create parent directories");
    }
    std::fs::write(path, vec![b'x'; size]).expect("failed to write fixture file");
}

fn fixture_root() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::create_dir_all(temp_dir.path().join("Applications/Demo.app/Contents"))
        .expect("failed to create mock app bundle");
    std::fs::write(
        temp_dir.path().join("Applications/Demo.app/Contents/Info.plist"),
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleIdentifier</key>
  <string>com.demo</string>
</dict>
</plist>
"#,
    )
    .expect("failed to write mock app bundle plist");
    write_fixture_file(temp_dir.path(), ".npm/_cacache/pkg.tgz", 2048);
    write_fixture_file(temp_dir.path(), ".cargo/registry/cache/serde.crate", 1024);
    write_fixture_file(temp_dir.path(), ".Trash/dead.txt", 512);
    write_fixture_file(temp_dir.path(), "Library/Caches/com.demo/cache.bin", 4096);
    write_fixture_file(
        temp_dir.path(),
        "Library/Application Support/com.demo/state.db",
        3072,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Application Support/com.old.app/state.db",
        2048,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Containers/com.demo/Documents/data.bin",
        2048,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Containers/com.old.app/Documents/data.bin",
        1024,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Containers/com.demo/Data/Caches/cache.bin",
        3072,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Containers/com.kingsoft.wpsoffice.mac/Data/Caches/cache.bin",
        1536,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Containers/com.docker.docker/Data/log/docker.log",
        1024,
    );
    write_fixture_file(
        temp_dir.path(),
        "Library/Containers/com.docker.docker/Data/vms/0/data.raw",
        2048,
    );
    write_fixture_file(temp_dir.path(), ".codex/state.json", 1024);
    write_fixture_file(temp_dir.path(), ".cursor/index.db", 1536);
    write_fixture_file(temp_dir.path(), ".kiro/session.log", 768);
    write_fixture_file(
        temp_dir.path(),
        "Documents/tauri-rDemo/target/debug/app",
        4096,
    );
    write_fixture_file(temp_dir.path(), "Documents/tauri-rDemo/dist/index.js", 2048);
    write_fixture_file(temp_dir.path(), "Documents/tauri-rDemo/npm-debug.log", 512);
    write_fixture_file(temp_dir.path(), ".rcleaner/rules.toml", 0);
    std::fs::write(
        temp_dir.path().join(".rcleaner/rules.toml"),
        r#"
version = 1

[[rules]]
id = "r-target"
title = "Rust target"
category = "R 系列构建缓存"
pattern = "~/Documents/tauri-r*/target"
enabled = true
cleanable = true
default_selected = false
risk = "medium"

[[rules]]
id = "r-log"
title = "debug log"
category = "R 系列调试文件"
pattern = "~/Documents/tauri-r*/npm-debug.log*"
enabled = true
cleanable = true
default_selected = false
risk = "low"
"#,
    )
    .expect("failed to write custom rules file");
    temp_dir
}

fn root_arg(root: &std::path::Path) -> String {
    root.to_string_lossy().to_string()
}

#[test]
fn info_json_returns_family_metadata() {
    let output = run_cli(&["info", "--json"]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["command"], "info");
    assert_eq!(payload["data"]["name"], "rCleaner");
    assert_eq!(payload["data"]["binary"], "rcleaner");
    assert_eq!(payload["data"]["architecture"], "simple-tool");
}

#[test]
fn capabilities_json_lists_scan_and_clean_commands() {
    let output = run_cli(&["capabilities", "--json"]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    let commands = payload["data"]["commands"]
        .as_array()
        .expect("commands should be an array");

    assert!(commands.iter().any(|item| item["command"] == "scan"));
    assert!(commands.iter().any(|item| item["command"] == "clean"));
}

#[test]
fn scan_json_reports_known_targets_under_custom_root() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&["scan", "--root", &root, "--json"]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["command"], "scan");
    assert!(payload["data"]["reclaimableBytes"].as_u64().unwrap_or(0) > 0);

    let targets = payload["data"]["targets"]
        .as_array()
        .expect("targets should be an array");

    let npm_target = targets
        .iter()
        .find(|item| item["id"] == "npm-download-cache")
        .expect("npm target should exist");
    assert_eq!(npm_target["exists"], true);
    assert!(npm_target["sizeBytes"].as_u64().unwrap_or(0) >= 2048);

    let app_cache_target = targets
        .iter()
        .find(|item| item["id"] == "app-cache:com.demo")
        .expect("dynamic app cache target should exist");
    assert_eq!(app_cache_target["cleanable"], true);

    let app_support_target = targets
        .iter()
        .find(|item| item["id"] == "app-support:com.demo")
        .expect("dynamic app support target should exist");
    assert_eq!(app_support_target["cleanable"], false);
    assert_eq!(app_support_target["expertCleanable"], false);

    let orphan_support_target = targets
        .iter()
        .find(|item| item["id"] == "app-support:com.old.app")
        .expect("orphan app support target should exist");
    assert_eq!(orphan_support_target["category"], "疑似应用残留");
    assert_eq!(orphan_support_target["cleanable"], false);
    assert_eq!(orphan_support_target["expertCleanable"], true);

    let app_container_cache_target = targets
        .iter()
        .find(|item| item["id"] == "app-container-cache:com.demo:Data-Caches")
        .expect("dynamic app container cache target should exist");
    assert_eq!(app_container_cache_target["cleanable"], true);

    let wps_profile_target = targets
        .iter()
        .find(|item| item["id"] == "profile:wps-caches")
        .expect("wps profile target should exist");
    assert_eq!(wps_profile_target["category"], "应用专属清理");
    assert_eq!(wps_profile_target["cleanable"], true);

    let docker_vm_target = targets
        .iter()
        .find(|item| item["id"] == "profile:docker-vms")
        .expect("docker vm profile target should exist");
    assert_eq!(docker_vm_target["category"], "应用专属概览");
    assert_eq!(docker_vm_target["cleanable"], false);

    let codex_target = targets
        .iter()
        .find(|item| item["id"] == "codex-home")
        .expect("codex target should exist");
    assert_eq!(codex_target["cleanable"], false);
    assert!(codex_target["sizeBytes"].as_u64().unwrap_or(0) >= 1024);

    let custom_target = targets
        .iter()
        .find(|item| {
            item["id"]
                .as_str()
                .unwrap_or("")
                .starts_with("custom:r-target:")
        })
        .expect("custom r-series target should exist");
    assert_eq!(custom_target["category"], "R 系列构建缓存");
}

#[test]
fn clean_json_dry_run_keeps_fixture_files() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "npm-download-cache",
        "--dry-run",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert_eq!(payload["data"]["dryRun"], true);

    let fixture_file = temp_dir.path().join(".npm/_cacache/pkg.tgz");
    assert!(fixture_file.exists(), "dry-run should not delete files");
}

#[test]
fn clean_json_removes_target_contents() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&["clean", "--target", "trash", "--root", &root, "--json"]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["freedBytes"].as_u64().unwrap_or(0) >= 512);

    let trash_dir = temp_dir.path().join(".Trash");
    let entries = std::fs::read_dir(trash_dir)
        .expect("trash dir should still exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to read trash dir");
    assert!(entries.is_empty(), "trash contents should be removed");
}

#[test]
fn clean_json_removes_dynamic_app_cache_contents() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "app-cache:com.demo",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["freedBytes"].as_u64().unwrap_or(0) >= 4096);

    let cache_dir = temp_dir.path().join("Library/Caches/com.demo");
    let entries = std::fs::read_dir(cache_dir)
        .expect("cache dir should still exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to read cache dir");
    assert!(
        entries.is_empty(),
        "dynamic cache contents should be removed"
    );
}

#[test]
fn clean_rejects_unknown_target() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "unknown-target",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(2));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "invalid_arguments");
}

#[test]
fn clean_rejects_read_only_dynamic_target() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "app-support:com.demo",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(2));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "invalid_arguments");
}

#[test]
fn clean_rejects_read_only_target_even_with_allow_readonly_flag() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "app-support:com.demo",
        "--allow-readonly",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(2));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], false);
    assert_eq!(payload["error"]["code"], "invalid_arguments");
}

#[test]
fn clean_allows_orphan_target_with_allow_readonly_flag() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "app-support:com.old.app",
        "--allow-readonly",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["freedBytes"].as_u64().unwrap_or(0) >= 2048);
}

#[test]
fn clean_json_removes_dynamic_container_cache_contents() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "app-container-cache:com.demo:Data-Caches",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["freedBytes"].as_u64().unwrap_or(0) >= 3072);

    let cache_dir = temp_dir
        .path()
        .join("Library/Containers/com.demo/Data/Caches");
    let entries = std::fs::read_dir(cache_dir)
        .expect("container cache dir should still exist")
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to read container cache dir");
    assert!(
        entries.is_empty(),
        "container cache contents should be removed"
    );
}

#[test]
fn clean_json_removes_app_profile_cache_contents() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let output = run_cli(&[
        "clean",
        "--target",
        "profile:wps-caches",
        "--root",
        &root,
        "--json",
    ]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["freedBytes"].as_u64().unwrap_or(0) >= 1536);
}

#[test]
fn clean_allows_custom_rule_target() {
    let temp_dir = fixture_root();
    let root = root_arg(temp_dir.path());

    let scan_output = run_cli(&["scan", "--root", &root, "--json"]);
    let scan_payload = parse_stdout_json(&scan_output);
    let targets = scan_payload["data"]["targets"]
        .as_array()
        .expect("targets should be an array");
    let custom_id = targets
        .iter()
        .find_map(|item| {
            let id = item["id"].as_str()?;
            if id.starts_with("custom:r-log:") {
                Some(id.to_string())
            } else {
                None
            }
        })
        .expect("custom rule target id should exist");

    let output = run_cli(&["clean", "--target", &custom_id, "--root", &root, "--json"]);
    assert_eq!(output.status.code(), Some(0));

    let payload = parse_stdout_json(&output);
    assert_eq!(payload["ok"], true);
    assert!(payload["data"]["freedBytes"].as_u64().unwrap_or(0) >= 512);
}
