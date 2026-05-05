// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    match rcleaner_lib::cli::run_from_env() {
        rcleaner_lib::cli::CliOutcome::LaunchDesktop => rcleaner_lib::run(),
        rcleaner_lib::cli::CliOutcome::Exit(code) => std::process::exit(code),
    }
}
