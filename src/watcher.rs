use crate::log;
use crate::command::ParsedCommand;
use crate::FYR;
use chrono::Utc;
use colored::*;
use notify::{Event, EventKind, RecursiveMode, Watcher, recommended_watcher};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use terminal_size::{Width, terminal_size};

pub fn start_watcher(
    paths: Vec<&Path>,
    command: ParsedCommand,
    run_str: &str,
    debounce: u64,
    quiet: bool,
    no_clear: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80)
        / 2;
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;

    log!(
        quiet,
        "{} watching — will run '{}' on changes",
        FYR.yellow(),
        run_str
    );

    for path in &paths {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }

    log!(quiet, "{}", "_".repeat(width));

    let mut last_run = Instant::now();
    let mut child = Some(
        Command::new(&command.cmd)
            .args(&command.args)
            .spawn()
            .expect("failed to spawn command"),
    );

    let (reaper_tx, reaper_rx) = mpsc::channel::<Child>();
    thread::spawn(move || {
        while let Ok(mut c) = reaper_rx.recv() {
            c.kill().ok();
            c.wait().ok();
        }
    });

    for event in rx {
        match event {
            Ok(e) if matches!(e.kind, EventKind::Modify(_) | EventKind::Create(_)) => {
                if last_run.elapsed() < Duration::from_millis(debounce) {
                    continue;
                }
                last_run = Instant::now();

                if let Some(c) = child.take() {
                    reaper_tx.send(c).ok();
                }
                let file_name = e
                    .paths
                    .first()
                    .and_then(|p| p.file_name())
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();

                if no_clear {
                    log!(quiet, "{}", "_".repeat(width));
                } else {
                    clearscreen::clear().unwrap();
                }
                log!(
                    quiet,
                    "{} {} changed at {}",
                    FYR.yellow(),
                    file_name.cyan(),
                    Utc::now().format("%H:%M:%S")
                );
                log!(quiet, "{}", "_".repeat(width));

                child = Some(
                    Command::new(&command.cmd)
                        .args(&command.args)
                        .spawn()
                        .expect("failed to spawn command"),
                );
            }
            Err(e) => eprintln!("{} watch error: {:#?}", "Error:".red(), e),
            _ => {}
        }
    }

    Ok(())
}
