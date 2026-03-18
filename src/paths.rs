use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{Result,anyhow};

pub fn validate_paths(paths: &[&Path], quiet: bool) -> Result<()> {
    log!(quiet, "checking paths...");
    for path in paths {
        if path.exists() {
            log!(
                quiet,
                "  {} {}",
                path.display().to_string().cyan(),
                "exists".green()
            );
        } else {
            return Err(anyhow!(" '{}' doesn't exist", path.display()));
        }
    }
    Ok(())
}

pub fn find_by_extensions(dirs: &[String], extensions: &[String]) -> Vec<PathBuf> {
    let roots: Vec<&str> = if dirs.is_empty() {
        vec!["."]
    } else {
        dirs.iter().map(|s| s.as_str()).collect()
    };

    roots.iter()
        .flat_map(|dir| {
            WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|e| e.path().to_path_buf())
                .filter(|p| {
                    p.extension()
                        .map(|e| extensions.iter().any(|ext| ext.as_str() == e))
                        .unwrap_or(false)
                })
        })
        .collect()
}

pub fn resolve_paths(watch: Vec<String>, extensions: Option<Vec<String>>) -> Vec<String> {
    match extensions {
        Some(exts) if !exts.is_empty() => {
            find_by_extensions(&watch, &exts)
                .iter()
                .map(|p| p.display().to_string())
                .collect()
        }
        _ => watch,
    }
}
