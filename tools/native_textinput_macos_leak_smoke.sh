#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_addr="127.0.0.1:8001"
target_name="makepad-example-native-text-input-macos-standalone-diag"
output="docs/native-textinput-evidence/macos-leak-runtime.md"
timeout=90
allow_no_clear=0
clear_ids=()
command_line="$0 $*"
self_test=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_macos_leak_smoke.sh [options]

Runs the diagnostic NativeTextInput macOS standalone Studio RunItem and verifies
the feature-gated live counter reaches zero before ClearBuild. On success it writes
docs/native-textinput-evidence/macos-leak-runtime.md.

Options:
  --studio ADDR          Studio bridge target, default 127.0.0.1:8001.
  --clear-build-id ID    Send ClearBuild for a previous build id before RunItem.
                         Repeat to clear multiple build tabs.
  --allow-no-clear       Allow RunItem without ClearBuild. Use only when
                         ListBuilds shows no old build for this target.
  --target NAME          RunItem name, default makepad-example-native-text-input-macos-standalone-diag.
  --output PATH          Evidence file to write on success.
  --timeout SECONDS      Startup/query timeout, default 90.
  --self-test            Test parser guards without connecting to Studio.
  -h, --help             Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --studio)
            studio_addr="${2:?missing value for --studio}"
            shift 2
            ;;
        --clear-build-id)
            clear_ids+=("${2:?missing value for --clear-build-id}")
            shift 2
            ;;
        --target)
            target_name="${2:?missing value for --target}"
            shift 2
            ;;
        --allow-no-clear)
            allow_no_clear=1
            shift
            ;;
        --output)
            output="${2:?missing value for --output}"
            shift 2
            ;;
        --timeout)
            timeout="${2:?missing value for --timeout}"
            shift 2
            ;;
        --self-test)
            self_test=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

extract_build_id() {
    local line="$1"
    if [[ "$line" =~ \"build_id\":\[([0-9]+)\] ]]; then
        printf '%s\n' "${BASH_REMATCH[1]}"
        return 0
    fi
    echo "could not parse build_id from: $line" >&2
    exit 1
}

is_app_connection_line() {
    local line="$1"
    local build_id="$2"
    printf '%s\n' "$line" | rg -q "\"AppStarted\".*\"build_id\":\\[$build_id\\]|\"RunViewCreated\".*\"build_id\":\\[$build_id\\]"
}

log_line_matches_live_count() {
    local line="$1"
    local count="$2"
    printf '%s\n' "$line" | rg -q "QueryLogResults.*NativeTextInput live count: $count"
}

run_self_test() {
    local build_line='{"BuildStarted":{"build_id":[88],"mount":"makepad","package":"makepad-example-native-text-input"}}'
    local app_line='{"AppStarted":{"build_id":[88]}}'
    local run_view_line='{"RunViewCreated":{"build_id":[88],"kind_id":0}}'
    local live_three='{"QueryLogResults":{"build_id":[88],"entries":["NativeTextInput live count: 3"],"done":true}}'
    local live_zero='{"QueryLogResults":{"build_id":[88],"entries":["NativeTextInput live count: 0"],"done":true}}'

    if [[ "$(extract_build_id "$build_line")" != "88" ]]; then
        echo "self-test failed: build id parser returned the wrong id" >&2
        exit 1
    fi
    if ! is_app_connection_line "$app_line" 88; then
        echo "self-test failed: AppStarted line was not accepted" >&2
        exit 1
    fi
    if ! is_app_connection_line "$run_view_line" 88; then
        echo "self-test failed: RunViewCreated line was not accepted" >&2
        exit 1
    fi
    if is_app_connection_line "$app_line" 87; then
        echo "self-test failed: AppStarted line matched the wrong build id" >&2
        exit 1
    fi
    if ! log_line_matches_live_count "$live_three" 3; then
        echo "self-test failed: live count 3 log was not accepted" >&2
        exit 1
    fi
    if ! log_line_matches_live_count "$live_zero" 0; then
        echo "self-test failed: live count 0 log was not accepted" >&2
        exit 1
    fi

    echo "NativeTextInput macOS leak self-test passed."
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

if [[ "${#clear_ids[@]}" -eq 0 && "$allow_no_clear" -eq 0 ]]; then
    echo "refusing to RunItem without --clear-build-id or --allow-no-clear" >&2
    echo "run tools/native_textinput_studio_gate.sh --list-builds first" >&2
    exit 2
fi

bridge="target/release/cargo-makepad"
if [[ ! -x "$bridge" ]]; then
    echo "$bridge is missing or not executable; build cargo-makepad release first" >&2
    exit 1
fi

log_file="$(mktemp)"
bridge_in_fifo="$(mktemp -u)"
bridge_out_fifo="$(mktemp -u)"
mkfifo "$bridge_in_fifo" "$bridge_out_fifo"
cleanup() {
    exec 3>&- || true
    exec 4<&- || true
    if [[ -n "${bridge_pid:-}" ]] && kill -0 "$bridge_pid" 2>/dev/null; then
        kill "$bridge_pid" 2>/dev/null || true
    fi
    rm -f "$log_file" "$bridge_in_fifo" "$bridge_out_fifo"
}
trap cleanup EXIT

exec 3<>"$bridge_in_fifo"
exec 4<>"$bridge_out_fifo"
"$bridge" studio --studio="$studio_addr" <"$bridge_in_fifo" >"$bridge_out_fifo" &
bridge_pid="$!"

send_json() {
    printf '%s\n' "$1" >&3
}

wait_for_line() {
    local pattern="$1"
    local label="$2"
    local deadline=$((SECONDS + timeout))
    local line

    while ((SECONDS < deadline)); do
        if IFS= read -r -t 1 line <&4; then
            printf '%s\n' "$line" >>"$log_file"
            if printf '%s\n' "$line" | rg -q "$pattern"; then
                printf '%s\n' "$line"
                return 0
            fi
            if printf '%s\n' "$line" | rg -q '"Error"'; then
                echo "Studio bridge error while waiting for $label:" >&2
                echo "$line" >&2
                exit 1
            fi
        elif ! kill -0 "$bridge_pid" 2>/dev/null; then
            echo "Studio bridge exited while waiting for $label" >&2
            cat "$log_file" >&2
            exit 1
        fi
    done

    echo "timed out waiting for $label" >&2
    cat "$log_file" >&2
    exit 1
}

query_logs_for() {
    local build_id="$1"
    local pattern="$2"
    local label="$3"
    local deadline=$((SECONDS + timeout))
    local line

    while ((SECONDS < deadline)); do
        send_json "{\"QueryLogs\":{\"build_id\":[$build_id],\"pattern\":\"$pattern\",\"live\":false}}"
        line="$(wait_for_line '"QueryLogResults"' "$label")"
        if printf '%s\n' "$line" | rg -q "$pattern"; then
            printf '%s\n' "$line"
            return 0
        fi
        sleep 1
    done

    echo "timed out waiting for QueryLogs pattern '$pattern' for $label" >&2
    cat "$log_file" >&2
    exit 1
}

send_json '{"ListBuilds":[]}'
wait_for_line '"Builds"' "ListBuilds response" >/dev/null

if [[ "${#clear_ids[@]}" -gt 0 ]]; then
    for id in "${clear_ids[@]}"; do
        send_json "{\"ClearBuild\":{\"build_id\":[$id]}}"
    done
fi

send_json "{\"RunItem\":{\"mount\":\"makepad\",\"name\":\"$target_name\"}}"
started_line="$(wait_for_line '"BuildStarted"|"AppStarted"|"RunViewCreated"' "diagnostic app startup")"
build_id="$(extract_build_id "$started_line")"

query_logs_for "$build_id" "NativeTextInput live count: 3" "live counter increment" >/dev/null
query_logs_for "$build_id" "NativeTextInput live count: 0" "live counter decrement" >/dev/null
send_json "{\"ClearBuild\":{\"build_id\":[$build_id]}}"

mkdir -p "$(dirname "$output")"
transcript="${output%.md}.log"
cp "$log_file" "$transcript"
cat >"$output" <<EOF
Status: PASS
RunItem: $target_name
Example: examples/native_text_input
Verified: ClearBuild RunItem detach leak

Studio: $studio_addr
BuildId: $build_id
Transcript: $transcript
Command: $command_line
Evidence:
- QueryLogs matched "NativeTextInput live count: 3" after diagnostic RunItem startup.
- QueryLogs matched "NativeTextInput live count: 0" after diagnostic in-process close.
- ClearBuild was sent for build_id [$build_id] after the leak counter returned to zero.
EOF

echo "wrote $output"
