use crate::log;
use crate::FYR;
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use walkdir::WalkDir;

pub fn validate_paths(paths: &[&Path], quiet: bool) {
    log!(quiet, "{} checking paths...", FYR.yellow());
    for path in paths {
        if path.exists() {
            log!(
                quiet,
                "  {} {}",
                path.display().to_string().cyan(),
                "exists".green()
            );
        } else {
            eprintln!("{} '{}' doesn't exist", "Error:".red(), path.display());
            process::exit(1);
        }
    }
}

pub fn find_by_extensions(extensions: &[String]) -> Vec<PathBuf> {
    WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.path().to_path_buf())
        .filter(|p| {
            p.extension()
                .map(|e| extensions.iter().any(|ext| ext.as_str() == e))
                .unwrap_or(false)
        })
        .collect()
}

pub fn resolve_paths(watch: Vec<String>, extensions: Option<Vec<String>>) -> Vec<String> {
    match extensions {
        Some(exts) if !exts.is_empty() => {
            let cache_path = std::env::temp_dir().join("fyr_path_cache.json");
            let dirs_mtime: u64 = watch
                .iter()
                .filter_map(|w| fs::metadata(w).ok())
                .filter_map(|m| m.modified().ok())
                .filter_map(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .sum();

            if let Ok(cached) = fs::read_to_string(&cache_path) {
                if let Ok((stored_mtime, stored_watch, stored_exts, paths)) =
                    serde_json::from_str::<(u64, Vec<String>, Vec<String>, Vec<String>)>(&cached)
                {
                    if stored_mtime == dirs_mtime && stored_watch == watch && stored_exts == exts {
                        return paths;
                    }
                }
            }

            let paths: Vec<String> = find_by_extensions(&exts)
                .iter()
                .map(|p| p.display().to_string())
                .collect();

            let _ = fs::write(
                &cache_path,
                serde_json::to_string(&(dirs_mtime, &watch, &exts, &paths)).unwrap_or_default(),
            );

            paths
        }
        _ => watch,
    }
}
