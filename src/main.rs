use clap::{Parser, Subcommand};
use colored::*;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process;

mod command;
mod config;
mod paths;
mod tasks;
mod templates;
mod watcher;

use command::{parse_command, validate_command};
use config::{load_config, resolve_config};
use paths::{resolve_paths, validate_paths};
use tasks::run_task;
use templates::get_template;

pub const FYR: &str = "[fyr]";
pub const DEBOUNCE_MS: u64 = 150;

#[macro_export]
macro_rules! log {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet {
            println!($($arg)*);
        }
    };
}

#[derive(Parser)]
#[command(
    name = "fyr",
    version,
    about = "A fast, minimal file watcher that runs a command every time you save."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(short, long, num_args = 1..)]
    watch: Vec<String>,
    #[arg(short, long)]
    run: Option<String>,
    #[arg(short, long, num_args = 1..)]
    extensions: Option<Vec<String>>,
    #[arg(long, short, default_value_t = DEBOUNCE_MS)]
    debounce: u64,
    #[arg(long, short)]
    global: bool,
    #[arg(long, short)]
    quiet: bool,
    #[arg(long, short)]
    no_clear: bool,
}

#[derive(Subcommand)]
enum Commands {
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    Run {
        name: Option<String>,
        #[arg(short, long, num_args = 1..)]
        watch: Option<Vec<String>>,
        #[arg(short, long)]
        run: Option<String>,
        #[arg(short, long, num_args = 1..)]
        extensions: Option<Vec<String>>,
        #[arg(long, short, default_value_t = DEBOUNCE_MS)]
        debounce: u64,
        #[arg(long, short)]
        global: bool,
        #[arg(long, short)]
        quiet: bool,
        #[arg(long, short)]
        no_clear: bool,
    },
    Init {
        template: Option<String>,
    },
}

#[derive(Subcommand)]
enum TaskAction {
    #[command(group = clap::ArgGroup::new("source").required(true).multiple(true))]
    Add {
        name: String,
        #[arg(short, long, num_args = 1.., group = "source")]
        watch: Vec<String>,
        #[arg(short, long)]
        run: String,
        #[arg(short, long, num_args = 1.., group = "source")]
        extensions: Option<Vec<String>>,
    },
    Remove {
        name: String,
    },
    List,
    #[command(group = clap::ArgGroup::new("edit_fields").required(true).multiple(true))]
    Edit {
        name: String,
        #[arg(short, long, num_args = 1.., group = "edit_fields")]
        watch: Vec<String>,
        #[arg(short, long, group = "edit_fields")]
        run: Option<String>,
        #[arg(short, long, num_args = 1.., group = "edit_fields")]
        extensions: Option<Vec<String>>,
    },
    Rename {
        name: String,
        new_name: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Task { action }) => {
            let mut config = load_config(true);
            match action {
                TaskAction::Add {
                    name,
                    watch,
                    run,
                    extensions,
                } => {
                    config.tasks.insert(
                        name.clone(),
                        config::Task {
                            watch,
                            run: Some(run),
                            extensions,
                        },
                    );
                    confy::store("fyr", None, config)?;
                    println!("{} task '{}' saved", FYR.yellow(), name);
                }
                TaskAction::Remove { name } => {
                    if config.tasks.remove(&name).is_some() {
                        confy::store("fyr", None, config)?;
                        println!("{} task '{}' removed", FYR.yellow(), name);
                    } else {
                        eprintln!("{} task '{}' not found", "Error:".red(), name);
                        process::exit(1);
                    }
                }
                TaskAction::List => {
                    if config.tasks.is_empty() {
                        println!("{} no saved tasks", FYR.yellow());
                    } else {
                        println!("{} saved tasks:", FYR.yellow());
                        for (name, task) in &config.tasks {
                            println!(
                                "  {} — watch: {:?} | extensions: {:?} | run: \"{}\"",
                                name.cyan(),
                                task.watch,
                                task.extensions,
                                task.run.as_deref().unwrap_or("none")
                            );
                        }
                    }
                }
                TaskAction::Edit {
                    name,
                    watch,
                    run,
                    extensions,
                } => {
                    let task = config.tasks.get_mut(&name).unwrap_or_else(|| {
                        eprintln!("{} task '{}' not found", "Error:".red(), name);
                        process::exit(1);
                    });
                    if let Some(x) = run {
                        task.run = Some(x);
                    }
                    if let Some(x) = extensions {
                        task.extensions = Some(x);
                    }
                    if !watch.is_empty() {
                        task.watch = watch;
                    }
                    confy::store("fyr", None, config)?;
                    println!("{} task '{}' updated", FYR.yellow(), name);
                }
                TaskAction::Rename { name, new_name } => {
                    let task = config.tasks.remove(&name).unwrap_or_else(|| {
                        eprintln!("{} task '{}' not found", "Error:".red(), name);
                        process::exit(1);
                    });
                    config.tasks.insert(new_name.clone(), task);
                    confy::store("fyr", None, config)?;
                    println!("{} task '{}' renamed to '{}'", FYR.yellow(), name, new_name);
                }
            }
        }

        Some(Commands::Run {
            name,
            watch,
            run,
            extensions,
            debounce,
            global,
            quiet,
            no_clear,
        }) => {
            let config = resolve_config(global, quiet);
            run_task(
                &config, name, watch, run, extensions, debounce, quiet, no_clear,
            )?;
        }

        None => {
            if args.watch.is_empty() && args.run.is_none() && args.extensions.is_none() {
                let config = if args.global {
                    log!(args.quiet, "{} loading global tasks", FYR.yellow());
                    load_config(true)
                } else if Path::new("fyr.toml").exists() {
                    log!(args.quiet, "{} loading tasks from 'fyr.toml'", FYR.yellow());
                    load_config(false)
                } else {
                    eprintln!(
                        "{} no 'fyr.toml' found — use -w/-e and -r to watch directly, or -g for global tasks",
                        "Error:".red()
                    );
                    process::exit(1);
                };
                run_task(
                    &config,
                    None,
                    None,
                    None,
                    None,
                    args.debounce,
                    args.quiet,
                    args.no_clear,
                )?;
            } else {
                if args.watch.is_empty() && args.extensions.is_none() {
                    eprintln!(
                        "{} please provide paths with -w or extensions with -e",
                        "Error:".red()
                    );
                    process::exit(1);
                }
                let run_str = args.run.unwrap_or_else(|| {
                    eprintln!("{} please provide a command with -r", "Error:".red());
                    process::exit(1);
                });
                let watch_strs = resolve_paths(args.watch, args.extensions);
                let paths: Vec<&std::path::Path> =
                    watch_strs.iter().map(|s| std::path::Path::new(s)).collect();
                let command = parse_command(&run_str);
                validate_paths(&paths, args.quiet);
                validate_command(&command, args.quiet);
                watcher::start_watcher(
                    paths,
                    command,
                    &run_str,
                    args.debounce,
                    args.quiet,
                    args.no_clear,
                )?;
            }
        }

        Some(Commands::Init { template }) => {
            let template_bytes = get_template(template);
            if Path::new("fyr.toml").exists() {
                log!(args.quiet, "{} fyr.toml already exists", FYR.yellow());
            } else {
                let mut file = File::create("fyr.toml")?;
                file.write_all(template_bytes)?;
                log!(
                    args.quiet,
                    "{} fyr.toml created — edit it then run fyr",
                    FYR.yellow()
                );
            }
        }
    }

    Ok(())
}
