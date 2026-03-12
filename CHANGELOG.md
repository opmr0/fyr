# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0] — 2026-03-12

### Added

#### Core File Watching
- File and directory watching via `notify` with recursive mode support
- Debounced event handling (default 150ms) to collapse rapid saves into a single clean run
- Automatic kill-and-restart of the previous process on each triggered change — no leftover processes
- Background reaper thread for safe, non-blocking child process cleanup
- Screen clearing between runs (opt-out with `--no-clear`)
- Timestamp displayed on each triggered run (`HH:MM:SS` via `chrono`)
- Terminal-width-aware separator lines for clean visual output

#### CLI Interface
- `fyr -w <paths> -r "<command>"` — inline watch + run mode
- `fyr -e <extensions> -r "<command>"` — watch by file extension, recursively from the current directory
- `fyr run [name]` — run a named task from config
- `fyr task add/remove/list/edit/rename` — full task management subcommands
- `fyr init [template]` — scaffold a `fyr.toml` from a blank or language-specific template
- `--debounce / -d` flag to customize the debounce window in milliseconds
- `--quiet / -q` flag to suppress all fyr log output (shows only command output)
- `--no-clear` flag to append runs instead of clearing the screen
- `--global / -g` flag to force loading of global tasks even when a local `fyr.toml` exists

#### Task System
- Named tasks stored globally via `confy` (platform-native config path)
- Each task stores watch paths, run command, and optional extension filters
- `default` field in config to auto-run a task without prompting
- Interactive task picker (via `dialoguer`) when no default is set and no name is provided
- Per-run overrides for watch paths, run command, and extensions without modifying the saved task
- `fyr task edit` supports partial updates — only the fields you specify are changed
- `fyr task rename` preserves all task data under a new name

#### Project Config (`fyr.toml`)
- Local `fyr.toml` support — loaded automatically when present in the current directory
- Config resolution order: local `fyr.toml` → global tasks → `--global` override
- `fyr init` creates a blank `fyr.toml` if one does not already exist
- 17 language templates via `fyr init <language>`: Rust, C, C++, Go, Zig, Swift, Haskell, Node.js, Ruby, PHP, Lua, Elixir, Java, Kotlin, CSS/SCSS, Shell

#### Path Handling
- Mixed watch input: accepts both files and directories in a single `-w` list
- Extension-based path discovery using `walkdir` for recursive file scanning
- Path cache stored in the system temp directory (`fyr_path_cache.json`) keyed by directory mtime, watch list, and extension list — invalidated automatically on change
- Startup path validation with clear error messages for missing paths
- Startup command validation using `which` to confirm the binary exists before watching

#### Developer Experience
- Colored terminal output via `colored` (green for success, cyan for paths/names, yellow for fyr prefix, red for errors)
- `[fyr]` prefix on all log lines for easy visual filtering
- Shell-aware command parsing via `shell_words` — flags in quoted commands are passed correctly to the subprocess
- `log!` macro respects `--quiet` throughout all modules
- `process::exit(1)` on all error paths for clean shell integration

---

[1.0.0]: https://codeberg.org/opmr0/fyr/releases/tag/v1.0.0
