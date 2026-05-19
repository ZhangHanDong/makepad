#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_addr="127.0.0.1:8001"
output="docs/native-textinput-evidence/macos-composition-runtime.md"
timeout=90
verified=""
allow_no_clear=0
clear_ids=()
target_name="makepad-example-native-text-input"
command_line="$0 $*"
self_test=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_macos_composition_evidence.sh [options]

Launches the checked-in NativeTextInput example through Studio RunItem and
writes macOS P1/P1.5/P4 composition evidence only after the operator explicitly
records the required behavior.

Options:
  --studio ADDR          Studio bridge target, default 127.0.0.1:8001.
  --clear-build-id ID    Send ClearBuild for a previous build id before RunItem.
                         Repeat to clear multiple build tabs.
  --allow-no-clear       Allow RunItem without ClearBuild. Use only when
                         ListBuilds shows no old build for this target.
  --verified TEXT        Required. Must contain:
                         shared_host clipped scroll z_order mount_queue same_frame
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
        --allow-no-clear)
            allow_no_clear=1
            shift
            ;;
        --verified)
            verified="${2:?missing value for --verified}"
            shift 2
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

extract_screenshot_path() {
    local line="$1"
    if [[ "$line" =~ \"path\"[[:space:]]*:[[:space:]]*\"([^\"]+)\" ]]; then
        printf '%s\n' "${BASH_REMATCH[1]}"
        return 0
    fi
    return 1
}

extract_widget_rect() {
    local widget_id="$1"
    local line="$2"
    if [[ "$line" =~ $widget_id[^0-9]*([0-9]+)[[:space:]]+([0-9]+)[[:space:]]+([0-9]+)[[:space:]]+([0-9]+) ]]; then
        printf '%s %s %s %s\n' "${BASH_REMATCH[1]}" "${BASH_REMATCH[2]}" "${BASH_REMATCH[3]}" "${BASH_REMATCH[4]}"
        return 0
    fi
    return 1
}

run_self_test() {
    local build_line='{"BuildStarted":{"build_id":[55],"mount":"makepad","package":"makepad-example-native-text-input"}}'
    local app_line='{"AppStarted":{"build_id":[55]}}'
    local run_view_line='{"RunViewCreated":{"build_id":[55],"kind_id":0}}'
    local screenshot_line='{"Screenshot":{"path":"/tmp/native-textinput-composition.png","width":560,"height":520}}'
    local widget_line='{"WidgetQuery":{"query":"id:set_both_button","result":"set_both_button 10 20 80 32"}}'

    if [[ "$(extract_build_id "$build_line")" != "55" ]]; then
        echo "self-test failed: build id parser returned the wrong id" >&2
        exit 1
    fi
    if ! is_app_connection_line "$app_line" 55; then
        echo "self-test failed: AppStarted line was not accepted" >&2
        exit 1
    fi
    if ! is_app_connection_line "$run_view_line" 55; then
        echo "self-test failed: RunViewCreated line was not accepted" >&2
        exit 1
    fi
    if is_app_connection_line "$app_line" 54; then
        echo "self-test failed: AppStarted line matched the wrong build id" >&2
        exit 1
    fi
    if [[ "$(extract_screenshot_path "$screenshot_line")" != "/tmp/native-textinput-composition.png" ]]; then
        echo "self-test failed: screenshot path parser returned the wrong path" >&2
        exit 1
    fi
    if [[ "$(extract_widget_rect set_both_button "$widget_line")" != "10 20 80 32" ]]; then
        echo "self-test failed: widget rect parser returned the wrong rect" >&2
        exit 1
    fi

    echo "NativeTextInput macOS composition self-test passed."
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

required_words=(shared_host clipped scroll z_order mount_queue same_frame)
if [[ -z "$verified" ]]; then
    echo "missing required --verified evidence words" >&2
    usage >&2
    exit 2
fi
for word in "${required_words[@]}"; do
    if ! printf '%s\n' "$verified" | rg -q "(^|[[:space:]])${word}([[:space:]]|$)"; then
        echo "--verified must contain word: $word" >&2
        exit 2
    fi
done

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

query_widget() {
    local build_id="$1"
    local widget_id="$2"
    send_json "{\"WidgetQuery\":{\"build_id\":[$build_id],\"query\":\"id:$widget_id\"}}"
    wait_for_line "\"WidgetQuery\".*$widget_id" "WidgetQuery id:$widget_id"
}

click_widget_center() {
    local build_id="$1"
    local widget_id="$2"
    local line x y w h cx cy

    line="$(query_widget "$build_id" "$widget_id")"
    if rect="$(extract_widget_rect "$widget_id" "$line")"; then
        read -r x y w h <<<"$rect"
    else
        echo "could not parse widget rect for $widget_id from: $line" >&2
        exit 1
    fi

    cx=$((x + w / 2))
    cy=$((y + h / 2))
    send_json "{\"Click\":{\"build_id\":[$build_id],\"x\":$cx,\"y\":$cy}}"
}

send_json '{"ListBuilds":[]}'
wait_for_line '"Builds"' "ListBuilds response" >/dev/null

for id in "${clear_ids[@]}"; do
    send_json "{\"ClearBuild\":{\"build_id\":[$id]}}"
done

send_json "{\"RunItem\":{\"mount\":\"makepad\",\"name\":\"$target_name\"}}"
started_line="$(wait_for_line '"BuildStarted"|"AppStarted"|"RunViewCreated"' "$target_name startup")"
build_id="$(extract_build_id "$started_line")"
if ! is_app_connection_line "$started_line" "$build_id"; then
    wait_for_line "\"AppStarted\".*\"build_id\":\\[$build_id\\]|\"RunViewCreated\".*\"build_id\":\\[$build_id\\]" "$target_name run view readiness" >/dev/null
fi

query_widget "$build_id" "native_input" >/dev/null
query_widget "$build_id" "native_label" >/dev/null
query_widget "$build_id" "clipped_native_input" >/dev/null
click_widget_center "$build_id" "set_both_button"
send_json "{\"Screenshot\":{\"build_id\":[$build_id]}}"
screenshot_line="$(wait_for_line '"Screenshot"' "composition screenshot")"
screenshot_path="$(extract_screenshot_path "$screenshot_line" || true)"
if [[ -z "$screenshot_path" || ! -f "$screenshot_path" ]]; then
    echo "Screenshot response did not provide a readable file path: $screenshot_line" >&2
    exit 1
fi

mkdir -p "$(dirname "$output")"
transcript="${output%.md}.log"
cp "$log_file" "$transcript"
cat >"$output" <<EOF
Status: PASS
RunItem: $target_name
Example: examples/native_text_input
Verified: $verified

Studio: $studio_addr
BuildId: $build_id
AfterScreenshot: $screenshot_path
Transcript: $transcript
Command: $command_line

Composition sequence:
- WidgetQuery id:native_input
- WidgetQuery id:native_label
- WidgetQuery id:clipped_native_input
- Click set_both_button
- Screenshot after same-frame native updates
EOF

echo "wrote $output"
