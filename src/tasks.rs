use crate::command::{parse_command, validate_command};
use crate::config::FyrConfig;
use crate::paths::{resolve_paths, validate_paths};
use crate::watcher::start_watcher;
use anyhow::{Context, Result, anyhow};
use dialoguer::Select;
use std::path::Path;

pub fn pick_task(config: &FyrConfig, name: Option<String>, quiet: bool) -> Result<String> {
    if let Some(n) = name {
        return Ok(n);
    }
    if let Some(d) = &config.default {
        log!(quiet, "default task '{}', running it", d);
        return Ok(d.clone());
    }
    let tasks: Vec<&String> = config.tasks.keys().collect();
    if tasks.is_empty() {
        return Err(anyhow!("no tasks found"));
    }
    let choice = Select::new()
        .with_prompt("which task do you want to run?")
        .items(&tasks)
        .interact()
        .context("cancelled")?;
    Ok(tasks[choice].to_string())
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
    no_ignore: bool,
) -> Result<()> {
    let name = pick_task(config, name, quiet)?;
    let task = config
        .tasks
        .get(&name)
        .cloned()
        .context(anyhow!("task '{}' not found", name))?;

    let extensions = extensions_override.or(task.extensions);
    let watch_strs = resolve_paths(watch_override.unwrap_or(task.watch), extensions);
    let run_str = run_override
        .or(task.run)
        .context(anyhow!("task has no run command, provide one with -r"))?;

    let paths: Vec<&Path> = watch_strs.iter().map(|s| Path::new(s)).collect();
    let command = parse_command(&run_str)?;
    validate_paths(&paths, quiet)?;
    validate_command(&command, quiet)?;
    start_watcher(paths, command, &run_str, debounce, quiet, no_clear,no_ignore)?;
    Ok(())
}
