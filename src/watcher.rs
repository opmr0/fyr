use crate::command::ParsedCommand;
use anyhow::{Context, Result};
use chrono::Local;
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use notify::{Event, EventKind, RecursiveMode, Watcher, recommended_watcher};
use std::path::Path;
use std::process::{Child, Command};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use terminal_size::{Width, terminal_size};

fn spawn_command(cmd: &str, args: &[String]) -> std::io::Result<Child> {
    #[cfg(windows)]
    return Command::new("cmd").args(["/C", cmd]).args(args).spawn();
    #[cfg(not(windows))]
    return Command::new(cmd).args(args).spawn();
}

pub fn start_watcher(
    paths: Vec<&Path>,
    command: ParsedCommand,
    run_str: &str,
    debounce: u64,
    quiet: bool,
    no_clear: bool,
    no_ignore: bool,
) -> Result<()> {
    let width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80)
        / 2;
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;

    log!(quiet, "watching, '{}' will run on changes", run_str);

    for path in &paths {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }

    println!("{}", "_".repeat(width));

    let mut last_run = Instant::now() - Duration::from_millis(debounce + 1);
    let mut child =
        Some(spawn_command(&command.cmd, &command.args).context("failed to spawn the command")?);

    let (reaper_tx, reaper_rx) = mpsc::channel::<Child>();
    thread::spawn(move || {
        while let Ok(mut c) = reaper_rx.recv() {
            c.kill().ok();
            c.wait().ok();
        }
    });

    let root = std::env::current_dir().unwrap();
    let mut builder = GitignoreBuilder::new(&root);
    builder.add(root.join(".gitignore"));
    let gitignore = builder.build().unwrap_or_else(|_| Gitignore::empty());

    for event in rx {
        match event {
            Ok(e) if matches!(e.kind, EventKind::Modify(_) | EventKind::Create(_)) => {
                if last_run.elapsed() < Duration::from_millis(debounce) {
                    continue;
                }
                last_run = Instant::now();

                let full_path = match e.paths.first() {
                    Some(p) => p.clone(),
                    None => continue,
                };

                if gitignore
                    .matched_path_or_any_parents(&full_path, full_path.is_dir())
                    .is_ignore() && !no_ignore
                {
                    continue;
                }

                if let Some(c) = child.take() {
                    reaper_tx.send(c).ok();
                }

                let file_name = full_path
                    .file_name()
                    .map(|f| f.to_string_lossy().to_string())
                    .unwrap_or_default();

                if no_clear {
                    log!(quiet, "{}", "_".repeat(width));
                } else {
                    clearscreen::clear().unwrap_or_else(|_| {
                        print!("\x1B[2J\x1B[1;1H");
                    });
                }
                log!(
                    quiet,
                    "{} changed at {}",
                    file_name.cyan(),
                    Local::now().format("%H:%M:%S")
                );
                println!("{}", "_".repeat(width));

                match spawn_command(&command.cmd, &command.args) {
                    Ok(c) => child = Some(c),
                    Err(e) => {
                        err!("failed to spawn '{}': {}", command.cmd, e);
                        continue;
                    }
                };
            }
            Err(e) => err!("watch error: {:#?}", e),
            _ => {}
        }
    }

    Ok(())
}