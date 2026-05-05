use crate::core::{
    app_identifier, app_name, binary_name, clean_targets, format_bytes, scan_targets, CleanOptions,
};
use clap::error::ErrorKind;
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::json;
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "rcleaner")]
#[command(about = "A lightweight resource manager for macOS caches and dev artifacts.")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Desktop,
    Info,
    Capabilities,
    Scan {
        #[arg(long, value_name = "PATH")]
        root: Option<PathBuf>,
    },
    Clean {
        #[arg(long = "target", value_name = "ID")]
        targets: Vec<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        allow_readonly: bool,
        #[arg(long, value_name = "PATH")]
        root: Option<PathBuf>,
    },
}

#[derive(Debug)]
pub enum CliOutcome {
    LaunchDesktop,
    Exit(i32),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    binary: &'static str,
    version: &'static str,
    identifier: &'static str,
    family: &'static str,
    architecture: &'static str,
    default_command: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlagInfo {
    flag: &'static str,
    description: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityInfo {
    command: &'static str,
    aliases: Vec<&'static str>,
    description: &'static str,
    json_supported: bool,
    reads_files: bool,
    writes_files: bool,
    examples: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CapabilityManifest {
    global_flags: Vec<FlagInfo>,
    commands: Vec<CapabilityInfo>,
}

fn app_info() -> AppInfo {
    AppInfo {
        name: app_name(),
        binary: binary_name(),
        version: env!("CARGO_PKG_VERSION"),
        identifier: app_identifier(),
        family: "r",
        architecture: "simple-tool",
        default_command: "desktop",
    }
}

fn capability_manifest() -> CapabilityManifest {
    CapabilityManifest {
        global_flags: vec![FlagInfo {
            flag: "--json",
            description: "Return a single machine-friendly JSON object.",
        }],
        commands: vec![
            CapabilityInfo {
                command: "desktop",
                aliases: vec![],
                description: "Launch the desktop workspace.",
                json_supported: false,
                reads_files: false,
                writes_files: false,
                examples: vec!["rcleaner desktop"],
            },
            CapabilityInfo {
                command: "info",
                aliases: vec![],
                description: "Show app identity and family metadata.",
                json_supported: true,
                reads_files: false,
                writes_files: false,
                examples: vec!["rcleaner info --json"],
            },
            CapabilityInfo {
                command: "capabilities",
                aliases: vec![],
                description: "List available CLI operations and usage hints.",
                json_supported: true,
                reads_files: false,
                writes_files: false,
                examples: vec!["rcleaner capabilities --json"],
            },
            CapabilityInfo {
                command: "scan",
                aliases: vec![],
                description:
                    "Scan built-in cleanup targets and app-level large directories (read-only by default).",
                json_supported: true,
                reads_files: true,
                writes_files: false,
                examples: vec![
                    "rcleaner scan --json",
                    "rcleaner scan --root /tmp/mock-home --json",
                ],
            },
            CapabilityInfo {
                command: "clean",
                aliases: vec![],
                description: "Clean targets; use --allow-readonly only when you intentionally accept high-risk cleanup.",
                json_supported: true,
                reads_files: true,
                writes_files: true,
                examples: vec![
                    "rcleaner clean --target npm-download-cache --json",
                    "rcleaner clean --all --dry-run --json",
                    "rcleaner clean --target app-support:com.demo --allow-readonly --json",
                ],
            },
        ],
    }
}

fn print_info(info: &AppInfo) {
    println!("{} {}", info.name, info.version);
    println!("binary: {}", info.binary);
    println!("identifier: {}", info.identifier);
    println!("family: {}", info.family);
    println!("architecture: {}", info.architecture);
    println!("default command: {}", info.default_command);
}

fn print_capabilities(manifest: &CapabilityManifest) {
    println!("global flags:");
    for flag in &manifest.global_flags {
        println!("  {}: {}", flag.flag, flag.description);
    }

    println!();
    println!("commands:");
    for command in &manifest.commands {
        let alias_suffix = if command.aliases.is_empty() {
            String::new()
        } else {
            format!(" (aliases: {})", command.aliases.join(", "))
        };
        println!("  {}{}", command.command, alias_suffix);
        println!("    {}", command.description);
        if let Some(example) = command.examples.first() {
            println!("    example: {example}");
        }
    }
}

fn print_scan_result(report: &crate::core::ScanReport) {
    println!("reclaimable: {}", report.reclaimable_label);
    println!(
        "disk: used {} / total {}",
        report.disk.used_label, report.disk.total_label
    );
    println!("targets: {}", report.target_count);
    println!();

    for target in report
        .targets
        .iter()
        .filter(|item| item.size_bytes > 0 || item.exists)
    {
        println!(
            "{} [{}] {}",
            target.title, target.category, target.size_label
        );
        println!("  {}", target.path);
        if let Some(error) = &target.scan_error {
            println!("  error: {error}");
        }
    }
}

fn print_clean_result(report: &crate::core::CleanReport) {
    println!("freed: {}", report.freed_label);
    println!("targets: {}", report.target_count);
    println!();

    for item in &report.targets {
        println!("{} -> {}", item.title, item.freed_label);
        println!("  {}", item.message);
    }
}

pub fn run_from_env() -> CliOutcome {
    let raw_args: Vec<_> = std::env::args_os().collect();
    let wants_json = raw_args.iter().any(|arg| arg == OsStr::new("--json"));
    let cli = match Cli::try_parse_from(raw_args) {
        Ok(cli) => cli,
        Err(error) => return emit_parse_error(wants_json, error),
    };

    match cli.command {
        None | Some(Commands::Desktop) => CliOutcome::LaunchDesktop,
        Some(Commands::Info) => {
            let info = app_info();
            emit_success(cli.json, "info", &info);
            if !cli.json {
                print_info(&info);
            }
            CliOutcome::Exit(0)
        }
        Some(Commands::Capabilities) => {
            let manifest = capability_manifest();
            emit_success(cli.json, "capabilities", &manifest);
            if !cli.json {
                print_capabilities(&manifest);
            }
            CliOutcome::Exit(0)
        }
        Some(Commands::Scan { root }) => match scan_targets(root) {
            Ok(report) => {
                emit_success(cli.json, "scan", &report);
                if !cli.json {
                    print_scan_result(&report);
                }
                CliOutcome::Exit(0)
            }
            Err(message) => emit_error(cli.json, "scan_failed", &message, 1),
        },
        Some(Commands::Clean {
            targets,
            all,
            dry_run,
            allow_readonly,
            root,
        }) => match clean_targets(CleanOptions {
            target_ids: targets,
            clean_all: all,
            dry_run,
            allow_readonly,
            root_override: root,
        }) {
            Ok(report) => {
                emit_success(cli.json, "clean", &report);
                if !cli.json {
                    print_clean_result(&report);
                }
                CliOutcome::Exit(0)
            }
            Err(message)
                if message.starts_with("unknown target id")
                    || message.starts_with("target id is read-only") =>
            {
                emit_error(cli.json, "invalid_arguments", &message, 2)
            }
            Err(message) if message.contains("--target") || message.contains("--all") => {
                emit_error(cli.json, "invalid_arguments", &message, 2)
            }
            Err(message) => emit_error(cli.json, "clean_failed", &message, 1),
        },
    }
}

fn emit_success<T: Serialize>(json_output: bool, command: &str, data: T) {
    if json_output {
        let payload = json!({
            "ok": true,
            "command": command,
            "data": data,
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    }
}

fn emit_error(json_output: bool, code: &str, message: &str, exit_code: i32) -> CliOutcome {
    if json_output {
        let payload = json!({
            "ok": false,
            "error": {
                "code": code,
                "message": message,
            }
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
    } else {
        eprintln!("{message}");
    }

    CliOutcome::Exit(exit_code)
}

fn emit_parse_error(json_output: bool, error: clap::Error) -> CliOutcome {
    let kind = error.kind();
    let message = error.to_string().trim().to_string();

    if json_output {
        if matches!(
            kind,
            ErrorKind::DisplayHelp | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        ) {
            let payload = json!({
                "ok": true,
                "command": "help",
                "data": {
                    "message": message,
                }
            });
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            return CliOutcome::Exit(0);
        }

        if kind == ErrorKind::DisplayVersion {
            let payload = json!({
                "ok": true,
                "command": "version",
                "data": {
                    "message": message,
                }
            });
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            return CliOutcome::Exit(0);
        }

        let payload = json!({
            "ok": false,
            "error": {
                "code": "invalid_arguments",
                "message": message,
            }
        });
        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
        return CliOutcome::Exit(2);
    }

    let exit_code = if matches!(
        kind,
        ErrorKind::DisplayHelp
            | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | ErrorKind::DisplayVersion
    ) {
        0
    } else {
        2
    };

    let _ = error.print();
    CliOutcome::Exit(exit_code)
}

pub fn print_target_choices() {
    let ids = crate::core::target_ids();
    if ids.is_empty() {
        return;
    }
    eprintln!("available targets:");
    for id in ids {
        eprintln!("  {}", id.to_string_lossy());
    }
}

pub fn human_freed_label(bytes: u64) -> String {
    format_bytes(bytes)
}
