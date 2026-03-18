use serde::{Deserialize, Serialize};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Default)]
pub struct FyrConfig {
    pub default: Option<String>,
    pub tasks: BTreeMap<String, Task>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub watch: Vec<String>,
    pub run: Option<String>,
    pub extensions: Option<Vec<String>>,
}

pub fn load_config(from_global: bool) -> Result<FyrConfig> {
    if from_global {
        confy::load::<FyrConfig>("fyr", None)
            .context("failed to read config")
    } else {
        let content = fs::read_to_string("fyr.toml")
            .context("failed to read fyr.toml")?;
        toml::from_str(&content)
            .context("invalid fyr.toml")
    }
}

pub fn resolve_config(global: bool, quiet: bool) -> Result<FyrConfig> {
    if global {
        log!(quiet, "loading global tasks");
        load_config(true)
    } else if Path::new("fyr.toml").exists() {
        log!(quiet, "loading tasks from 'fyr.toml'");
        load_config(false)
    } else {
        log!(quiet, "loading global tasks");
        load_config(true)
    }
}
