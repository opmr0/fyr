#!/usr/bin/env bash

TMPDIR=$(mktemp -d)
REPORT="$(pwd)/filewatcher_benchmark_report.txt"

cleanup() {
    rm -rf "$TMPDIR"
    for exe in fyr.exe watchexec.exe nodemon.cmd chokidar.cmd; do
        taskkill //F //IM "$exe" > /dev/null 2>&1 || true
    done
}
trap cleanup EXIT

WATCH_FILE="$TMPDIR/watch.txt"
WATCH_DIR="$TMPDIR/watchdir"
mkdir -p "$WATCH_DIR"
touch "$WATCH_FILE"

now_ms() {
    powershell -NoProfile -Command "[long](([DateTime]::UtcNow - [DateTime]::new(1970,1,1,0,0,0,[DateTimeKind]::Utc)).TotalMilliseconds)"
}

get_pid_mem_kb() {
    powershell -NoProfile -Command "
        try {
            \$p = Get-Process -Id $1 -ErrorAction Stop
            [math]::Round(\$p.WorkingSet64 / 1024)
        } catch { 0 }
    " 2>/dev/null | tr -d '\r\n '
}

get_pid_cpu() {
    powershell -NoProfile -Command "
        try {
            \$c1 = (Get-Process -Id $1 -ErrorAction Stop).CPU
            Start-Sleep -Milliseconds 500
            \$c2 = (Get-Process -Id $1 -ErrorAction Stop).CPU
            [math]::Round((\$c2 - \$c1) / 0.5 * 100) / 100
        } catch { 0 }
    " 2>/dev/null | tr -d '\r\n '
}

sample_process() {
    local pid="$1" duration="$2"
    local samples=0 total_mem=0 peak_mem=0 total_cpu=0
    local end_time=$(( $(date +%s) + duration ))
    while [ $(date +%s) -lt $end_time ]; do
        local mem cpu
        mem=$(get_pid_mem_kb "$pid")
        cpu=$(get_pid_cpu "$pid")
        [ -z "$mem" ] || [ "$mem" = "0" ] && { sleep 0.5; continue; }
        total_mem=$(( total_mem + mem ))
        [ "$mem" -gt "$peak_mem" ] && peak_mem=$mem
        total_cpu=$(awk "BEGIN{print $total_cpu + ${cpu:-0}}")
        samples=$(( samples + 1 ))
    done
    [ "$samples" -eq 0 ] && echo "0 0 0 0" && return
    echo "$(( total_mem / samples )) $peak_mem $(awk "BEGIN{printf \"%.2f\", $total_cpu / $samples}") $samples"
}

get_exe_pid() {
    tasklist 2>/dev/null | grep -i "$1" | awk '{print $2}' | head -1
}

print_header() {
    echo ""
    echo "══════════════════════════════════════════════════════"
    echo "  $1"
    echo "══════════════════════════════════════════════════════"
}

bench_startup() {
    local name="$1"; shift
    local cmd=("$@")
    local trials=20 total_ms=0 min_ms=99999
    for i in $(seq 1 $trials); do
        local start end elapsed
        start=$(now_ms)
        "${cmd[@]}" > /dev/null 2>&1
        end=$(now_ms)
        elapsed=$(( end - start ))
        total_ms=$(( total_ms + elapsed ))
        [ $elapsed -lt $min_ms ] && min_ms=$elapsed
    done
    local avg_ms
    avg_ms=$(awk "BEGIN{printf \"%.1f\", $total_ms / $trials}")
    echo "  $name — avg: ${avg_ms}ms  min: ${min_ms}ms"
}

bench_idle_mem() {
    local name="$1" exe_name="$2"; shift 2
    local cmd=("$@")
    "${cmd[@]}" > /dev/null 2>&1 &
    sleep 1.0
    local pid
    pid=$(get_exe_pid "$exe_name")
    [ -z "$pid" ] && echo "  $name — could not find process" && return
    local avg_mem peak_mem avg_cpu samples
    read avg_mem peak_mem avg_cpu samples <<< $(sample_process "$pid" 5)
    taskkill //F //PID "$pid" > /dev/null 2>&1 || true
    echo "  $name — mem: ${avg_mem}KB ($(awk "BEGIN{printf \"%.1f\", $avg_mem/1024}")MB)  peak: ${peak_mem}KB  cpu: ${avg_cpu}%"
}

bench_load() {
    local name="$1" exe_name="$2" out_token="$3"; shift 3
    local cmd=("$@")
    local out="$TMPDIR/load_${name}.txt"
    "${cmd[@]}" > "$out" 2>&1 &
    sleep 1.0
    local pid
    pid=$(get_exe_pid "$exe_name")
    [ -z "$pid" ] && echo "  $name — could not find process" && return
    local start end
    start=$(now_ms)
    for i in $(seq 1 50); do
        echo "$i" >> "$WATCH_FILE"
        sleep 0.05
    done
    end=$(now_ms)
    local avg_mem peak_mem avg_cpu samples
    read avg_mem peak_mem avg_cpu samples <<< $(sample_process "$pid" 3)
    taskkill //F //PID "$pid" > /dev/null 2>&1 || true
    local elapsed_s fires
    elapsed_s=$(awk "BEGIN{printf \"%.2f\", ($end - $start) / 1000}")
    fires=$(grep -c "$out_token" "$out" 2>/dev/null || echo 0)
    echo "  $name — fired: ${fires}/50 in ${elapsed_s}s  mem: ${avg_mem}KB  cpu: ${avg_cpu}%"
}

TOOLS=()
TOOL_LABELS=()

fyr_BIN="${fyr_BIN:-fyr}"
if command -v "$fyr_BIN" &>/dev/null; then
    TOOLS+=("fyr")
    TOOL_LABELS+=("fyr")
fi

if command -v watchexec &>/dev/null; then
    TOOLS+=("watchexec")
    TOOL_LABELS+=("watchexec")
fi

if command -v nodemon &>/dev/null; then
    TOOLS+=("nodemon")
    TOOL_LABELS+=("nodemon")
fi

if command -v chokidar &>/dev/null; then
    TOOLS+=("chokidar")
    TOOL_LABELS+=("chokidar")
fi

> "$REPORT"

run() {
    echo "File Watcher Comparative Benchmark"
    echo "Date   : $(date)"
    echo "OS     : $(uname -srm)"
    echo "CPU    : $(powershell -NoProfile -Command "(Get-WmiObject Win32_Processor).Name" 2>/dev/null | tr -d '\r')"
    echo "RAM    : $(powershell -NoProfile -Command "[math]::Round((Get-WmiObject Win32_ComputerSystem).TotalPhysicalMemory/1MB)" 2>/dev/null | tr -d '\r') MB"
    echo ""
    echo "Tools detected: ${TOOLS[*]}"

    print_header "BENCHMARK 1: Startup latency"
    [ -n "$(command -v $fyr_BIN)" ] && bench_startup "fyr      " "$fyr_BIN" task list
    [ -n "$(command -v watchexec)" ] && bench_startup "watchexec" watchexec --version
    [ -n "$(command -v nodemon)"   ] && bench_startup "nodemon  " nodemon --version
    [ -n "$(command -v chokidar)"  ] && bench_startup "chokidar " chokidar --version

    print_header "BENCHMARK 2: Idle memory and CPU"
    [ -n "$(command -v $fyr_BIN)" ] && bench_idle_mem "fyr      " "fyr.exe"       "$fyr_BIN" -w "$WATCH_FILE" -r "echo x"
    [ -n "$(command -v watchexec)" ] && bench_idle_mem "watchexec" "watchexec.exe" watchexec -w "$WATCH_FILE" -- echo x
    [ -n "$(command -v nodemon)"   ] && bench_idle_mem "nodemon  " "node.exe"      nodemon --watch "$WATCH_FILE" --exec "echo x"
    [ -n "$(command -v chokidar)"  ] && bench_idle_mem "chokidar " "node.exe"      chokidar "$WATCH_FILE" -c "echo x"

    print_header "BENCHMARK 3: Commands fired under load (50 changes)"
    touch "$WATCH_FILE"
    [ -n "$(command -v $fyr_BIN)" ] && bench_load "fyr      " "fyr.exe"       "triggered" "$fyr_BIN" -w "$WATCH_FILE" -r "echo triggered"
    touch "$WATCH_FILE"
    [ -n "$(command -v watchexec)" ] && bench_load "watchexec" "watchexec.exe" "triggered" watchexec -w "$WATCH_FILE" -- echo triggered
    touch "$WATCH_FILE"
    [ -n "$(command -v nodemon)"   ] && bench_load "nodemon  " "node.exe"      "triggered" nodemon --watch "$WATCH_FILE" --exec "echo triggered"
    touch "$WATCH_FILE"
    [ -n "$(command -v chokidar)"  ] && bench_load "chokidar " "node.exe"      "triggered" chokidar "$WATCH_FILE" -c "echo triggered"

    print_header "BENCHMARK 4: Version info"
    [ -n "$(command -v $fyr_BIN)" ] && echo "  fyr       : $($fyr_BIN --version 2>/dev/null)"
    [ -n "$(command -v watchexec)" ] && echo "  watchexec : $(watchexec --version 2>/dev/null)"
    [ -n "$(command -v nodemon)"   ] && echo "  nodemon   : $(nodemon --version 2>/dev/null)"
    [ -n "$(command -v chokidar)"  ] && echo "  chokidar  : $(chokidar --version 2>/dev/null)"

    print_header "SUMMARY TABLE"
    printf "  %-12s %-20s %-20s %-15s\n" "Tool" "Startup (avg)" "Idle Memory" "Load fires/50"
    echo "  ────────────────────────────────────────────────────────────────"
    echo "  (see individual benchmarks above for full numbers)"
    echo ""
    echo "Report saved to: $REPORT"
}

run | tee "$REPORT"