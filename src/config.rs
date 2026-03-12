use crate::log;
use crate::FYR;
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process;

#[derive(Serialize, Deserialize, Default)]
pub struct FyrConfig {
    pub default: Option<String>,
    pub tasks: HashMap<String, Task>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub watch: Vec<String>,
    pub run: Option<String>,
    pub extensions: Option<Vec<String>>,
}

pub fn load_config(from_global: bool) -> FyrConfig {
    if from_global {
        confy::load::<FyrConfig>("fyr", None).unwrap_or_else(|_| {
            eprintln!("{} failed to read config", "Error:".red());
            process::exit(1);
        })
    } else {
        let content = fs::read_to_string("fyr.toml").unwrap_or_else(|_| {
            eprintln!("{} failed to read fyr.toml", "Error:".red());
            process::exit(1);
        });
        toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!("{} invalid fyr.toml: {}", "Error:".red(), e);
            process::exit(1);
        })
    }
}

pub fn resolve_config(global: bool, quiet: bool) -> FyrConfig {
    if global {
        log!(quiet, "{} loading global tasks", FYR.yellow());
        load_config(true)
    } else if Path::new("fyr.toml").exists() {
        log!(quiet, "{} loading tasks from 'fyr.toml'", FYR.yellow());
        load_config(false)
    } else {
        log!(quiet, "{} loading global tasks", FYR.yellow());
        load_config(true)
    }
}
