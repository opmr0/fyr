use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[macro_use]
mod macros;
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

pub const DEBOUNCE_MS: u64 = 150;

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
    #[arg(long)]
    no_ignore: bool,
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
        #[arg(long)]
        no_ignore: bool,
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
    Default {
        name: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Task { action }) => {
            let mut config = load_config(true)?;
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
                    log!(false, "task '{}' saved", name);
                }
                TaskAction::Remove { name } => {
                    if config.tasks.remove(&name).is_some() {
                        confy::store("fyr", None, config)?;
                        log!(false, "task '{}' removed", name);
                    } else {
                        return Err(anyhow!("task '{}' not found", name));
                    }
                }
                TaskAction::List => {
                    if config.tasks.is_empty() {
                        log!(false, "no saved tasks");
                    } else {
                        log!(false, "saved tasks:");
                        for (name, task) in &config.tasks {
                            log!(
                                false,
                                "  {} --- watch: {:?} | extensions: {:?} | run: \"{}\"",
                                name.cyan(),
                                task.watch,
                                task.extensions.clone().unwrap_or(Vec::new()),
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
                    let task = config
                        .tasks
                        .get_mut(&name)
                        .context(anyhow!("task '{}' not found", name))?;
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
                    log!(false, "task '{}' updated", name);
                }
                TaskAction::Rename { name, new_name } => {
                    let task = config
                        .tasks
                        .remove(&name)
                        .context(anyhow!("task '{}' not found", name))?;
                    config.tasks.insert(new_name.clone(), task);
                    confy::store("fyr", None, config)?;
                    log!(false, "task '{}' renamed to '{}'", name, new_name);
                }
                TaskAction::Default { name } => {
                    if !config.tasks.contains_key(&name) {
                        return Err(anyhow!("task '{}' not found", name));
                    }
                    config.default = Some(name.clone());
                    confy::store("fyr", None, config)?;
                    log!(false, "default task set to '{}'", name);
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
            no_ignore
        }) => {
            let config = resolve_config(global, quiet)?;
            run_task(
                &config, name, watch, run, extensions, debounce, quiet, no_clear,no_ignore
            )?;
        }

        None => {
            if args.watch.is_empty() && args.run.is_none() && args.extensions.is_none() {
                let config = if args.global {
                    log!(args.quiet, "loading global tasks");
                    load_config(true)
                } else if Path::new("fyr.toml").exists() {
                    log!(args.quiet, "loading tasks from 'fyr.toml'");
                    load_config(false)
                } else {
                    return Err(anyhow!(
                        "no 'fyr.toml' found, use -w/-e and -r to watch directly, or -g for global tasks",
                    ));
                }?;
                run_task(
                    &config,
                    None,
                    None,
                    None,
                    None,
                    args.debounce,
                    args.quiet,
                    args.no_clear,
                    args.no_ignore
                )?;
            } else {
                if args.watch.is_empty() && args.extensions.is_none() {
                    return Err(anyhow!(
                        "please provide paths with -w or extensions with -e"
                    ));
                }
                let run_str = args.run.context("please provide a command with -r")?;
                let watch_strs = resolve_paths(args.watch, args.extensions);
                let paths: Vec<&std::path::Path> =
                    watch_strs.iter().map(|s| std::path::Path::new(s)).collect();
                let command = parse_command(&run_str)?;
                validate_paths(&paths, args.quiet)?;
                validate_command(&command, args.quiet)?;
                watcher::start_watcher(
                    paths,
                    command,
                    &run_str,
                    args.debounce,
                    args.quiet,
                    args.no_clear,
                    args.no_ignore
                )?;
            }
        }

        Some(Commands::Init { template }) => {
            let template_bytes = get_template(template);
            if Path::new("fyr.toml").exists() {
                log!(args.quiet, "fyr.toml already exists");
            } else {
                let mut file = File::create("fyr.toml")?;
                file.write_all(template_bytes)?;
                log!(args.quiet, "fyr.toml created, edit it then run fyr",);
            }
        }
    }

    Ok(())
}
