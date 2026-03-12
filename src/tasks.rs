use crate::log;
use crate::command::{parse_command, validate_command};
use crate::config::FyrConfig;
use crate::paths::{resolve_paths, validate_paths};
use crate::watcher::start_watcher;
use crate::FYR;
use colored::*;
use dialoguer::Select;
use std::path::Path;
use std::process;

pub fn pick_task(config: &FyrConfig, name: Option<String>, quiet: bool) -> String {
    if let Some(n) = name {
        return n;
    }
    if let Some(d) = &config.default {
        log!(quiet, "{} default task '{}' — running it", FYR.yellow(), d);
        return d.clone();
    }
    let tasks: Vec<&String> = config.tasks.keys().collect();
    let choice = Select::new()
        .with_prompt("which task do you want to run?")
        .items(&tasks)
        .interact()
        .unwrap_or_else(|_| {
            eprintln!("{} cancelled", "Error:".red());
            process::exit(1);
        });
    tasks[choice].to_string()
}

pub fn run_task(
    config: &FyrConfig,
    name: Option<String>,
    watch_override: Option<Vec<String>>,
    run_override: Option<String>,
    extensions_override: Option<Vec<String>>,
    debounce: u64,
    quiet: bool,
    no_clear: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let name = pick_task(config, name, quiet);
    let task = config.tasks.get(&name).cloned().unwrap_or_else(|| {
        eprintln!("{} task '{}' not found", "Error:".red(), name);
        process::exit(1);
    });

    let extensions = extensions_override.or(task.extensions);
    let watch_strs = resolve_paths(watch_override.unwrap_or(task.watch), extensions);
    let run_str = run_override.or(task.run).unwrap_or_else(|| {
        eprintln!(
            "{} task has no run command — provide one with -r",
            "Error:".red()
        );
        process::exit(1);
    });

    let paths: Vec<&Path> = watch_strs.iter().map(|s| Path::new(s)).collect();
    let command = parse_command(&run_str);
    validate_paths(&paths, quiet);
    validate_command(&command, quiet);
    start_watcher(paths, command, &run_str, debounce, quiet, no_clear)
}
