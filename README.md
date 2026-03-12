<img src="./assets/logo.png" width="100"/>

<br/>

# fyr

Watch files. Run commands. Stay in flow.

[![Crates.io](https://img.shields.io/crates/v/fyr)](https://crates.io/crates/fyr)
[![Downloads](https://img.shields.io/crates/d/fyr)](https://crates.io/crates/fyr)
[![License](https://img.shields.io/crates/l/fyr)](LICENSE)
[![Build](https://github.com/opmr0/fyr/actions/workflows/release.yml/badge.svg)](https://github.com/opmr0/fyr/actions)
[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org)

---

fyr runs a command every time you save a file — debounced, clean, and instant. No duplicate runs, no leftover processes, no noise.

- **Tasks** — save any watch+command pair by name and run it from anywhere with `fyr run <name>`
- **Project config** — drop a `fyr.toml` in your repo and commit your setup alongside your code
- **Language templates** — `fyr init rust`, `fyr init go`, and 14 more to get started in seconds
- **Zero-config mode** — run `fyr` with no arguments and pick a task from an interactive menu
- **Quiet mode** — strip fyr's output entirely so only your command speaks

---

## Installation

**macOS / Linux**

```bash
curl -sSf https://raw.githubusercontent.com/opmr0/fyr/main/install.sh | sh
```

**Windows (PowerShell)**

```powershell
iwr https://raw.githubusercontent.com/opmr0/fyr/main/install.ps1 -UseBasicParsing | iex
```

**Via cargo**

```bash
cargo install fyr
```
**From source**

```bash
cargo install --path .
```

---

## Quick Start

```bash
fyr -w src -r "cargo run"
```

Every time you save a file inside `src`, fyr runs `cargo run`. That's it.

---

## Watching Files

```bash
fyr -w <files or dirs> -r "<command>"
```

```bash
fyr -w src -r "cargo run"
fyr -w src tests -r "cargo test"
fyr -w main.go -r "go run main.go"
```

> **Tip:** Always wrap your command in quotes so its flags go to your command, not to fyr.

### Watch by extension

Watch all files with a given extension, recursively from the current directory.

```bash
fyr -e rs -r "cargo run"
fyr -e js ts -r "node index.js"
```

### Flags

| Flag           | Short | Description                          |
| -------------- | ----- | ------------------------------------ |
| `--watch`      | `-w`  | Files or directories to watch        |
| `--run`        | `-r`  | Command to run on change             |
| `--extensions` | `-e`  | Watch files by extension             |
| `--debounce`   | `-d`  | Debounce window in ms (default: 150) |
| `--quiet`      | `-q`  | Suppress fyr's own log output        |
| `--no-clear`   | —     | Don't clear the screen between runs  |

---

## Tasks

Save a watch + command pair as a named task and run it from anywhere with one word.

```bash
fyr task add build -w src -r "cargo build --release"
fyr task add test -w src tests -r "cargo test"

fyr run build
fyr run test
```

Tasks are stored globally and available from any directory.

### Task commands

```bash
fyr task add <name> -w <paths> -r "<command>"   # save a task
fyr task list                                    # list all tasks
fyr task edit <name> -w <new paths>             # update watch paths
fyr task edit <name> -r "<new command>"         # update command
fyr task rename <name> <new_name>               # rename a task
fyr task remove <name>                          # delete a task
```

You can also provide `-e` / `--extensions` when adding or editing a task. You must provide `-w`, `-e`, or both.

### Override on run

Run a task with a different path or command without permanently changing it:

```bash
fyr run build -w src/main.rs
fyr run build -r "cargo build"
```

### Run flags

| Flag         | Short | Description                                |
| ------------ | ----- | ------------------------------------------ |
| `--watch`    | `-w`  | Override watch paths                       |
| `--run`      | `-r`  | Override command                           |
| `--debounce` | `-d`  | Debounce window in ms                      |
| `--global`   | `-g`  | Use global tasks even if `fyr.toml` exists |
| `--quiet`    | `-q`  | Suppress fyr's own log output              |
| `--no-clear` | —     | Don't clear the screen between runs        |

---

## Project Config

`fyr` looks for a `fyr.toml` in your current directory first. If found, tasks are loaded from it instead of your global tasks — useful for committing your fyr setup alongside your project.

```bash
fyr init           # create a blank fyr.toml
fyr init rust      # create one from a language template
```

**Supported templates:** Rust, C, C++, Go, Zig, Swift, Haskell, Node.js, Ruby, PHP, Lua, Elixir, Java, Kotlin, CSS/SCSS, Shell

### fyr.toml format

```toml
default = "build"

[tasks.build]
watch = ["src"]
run = "cargo build --release"

[tasks.test]
watch = ["src", "tests"]
run = "cargo test"
```

### Config resolution

| Situation                          | What fyr loads          |
| ---------------------------------- | ----------------------- |
| `fyr.toml` exists in current dir   | Local tasks             |
| No `fyr.toml`                      | Global tasks            |
| `--global` / `-g` flag             | Global tasks (always)   |

---

## Zero-Config Mode

Run `fyr` or `fyr run` with no arguments. If a `fyr.toml` exists, fyr loads it and either runs the default task immediately or shows an interactive picker.

```bash
fyr
```

```
[fyr] loading tasks from 'fyr.toml'
[fyr] default task 'build' — running it
```

```
[fyr] loading tasks from 'fyr.toml'
? which task do you want to run?
> build
  test
  lint
```

Use `-g` to skip `fyr.toml` and always load global tasks:

```bash
fyr --global
```

---

## Quiet Mode

Suppresses fyr's own output and shows only your command's output. Works with `fyr`, `fyr run`, and `fyr run <name>`. Does not suppress errors or task management output.

```bash
fyr -w src -r "cargo run" -q
fyr run build -q
```

---

## Debounce

Editors often write to disk multiple times on a single save. fyr waits **150ms** after the last detected change before running — so you always get exactly one run per save, no matter how fast you type.

```bash
fyr -w src -r "cargo build" -d 500   # wait 500ms instead
```

---

## Benchmarks

Benchmarked on an Intel i7-9850H against the most popular file watchers.

| Tool          | Startup    | Idle Memory | Commands fired (50 rapid changes) |
| ------------- | ---------- | ----------- | --------------------------------- |
| **fyr**       | **219ms**  | **7.6 MB**  | 27/50 ¹                           |
| watchexec     | 238ms      | 13.5 MB     | 51/50                             |
| chokidar      | 501ms      | 37.6 MB     | 1/50 ²                            |
| nodemon       | 528ms      | 41.2 MB     | 102/50 ³                          |

Versions tested: fyr v1.0.0, watchexec v2.5.0, chokidar v3.6.0, nodemon v3.1.14

¹ Intentional — fyr debounces and kills stale runs, so rapid saves collapse into one clean run per burst. Adjust with `-d`.
² chokidar's debounce is too aggressive for rapid changes, causing it to miss most events.
³ nodemon fires duplicate events per change.

```bash
bash ./benchmark.sh   # run it yourself
```

Requires `watchexec`, `nodemon`, or `chokidar` for comparison. The script auto-detects what's installed and skips the rest.

---

## How It Works

1. fyr starts watching the paths you provide
2. A file changes — fyr waits for the debounce window to pass
3. If the previous command is still running, fyr kills it
4. fyr runs your command fresh

---

## Contributing

Found a bug or have an idea? Open an issue or submit a pull request.

---

## License

MIT — [LICENSE](LICENSE)