#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_addr="127.0.0.1:8001"
target_name="makepad-example-native-text-input"
list_only=0
print_jsonl=0
once=0
allow_no_clear=0
self_test=0
clear_ids=()

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_studio_gate.sh [options]

Starts the Makepad Studio remote bridge and sends compact JSONL control
packets for the NativeTextInput runtime gate. This script never launches the UI
with raw cargo run; it only uses Studio RunItem through the bridge.

Common flows:
  # 1. Inspect current builds first.
  tools/native_textinput_studio_gate.sh --list-builds

  # 2. Clear an old build id, then launch the normal smoke example.
  tools/native_textinput_studio_gate.sh --clear-build-id 6

  # 3. Launch the leak-counter diagnostic RunItem.
  tools/native_textinput_studio_gate.sh --diag --clear-build-id 6

Options:
  --studio ADDR          Studio bridge target, default 127.0.0.1:8001.
  --target NAME          RunItem name, default makepad-example-native-text-input.
  --diag                 Use makepad-example-native-text-input-diag.
  --list-builds          Only send ListBuilds.
  --clear-build-id ID    Send ClearBuild for a previous build id before RunItem.
                         Repeat to clear multiple build tabs.
  --allow-no-clear       Allow RunItem without ClearBuild. Use only when
                         ListBuilds shows no old build for this target.
  --print-jsonl          Print JSONL instead of starting the bridge.
  --once                 Do not keep stdin attached after sending initial JSONL.
  --self-test            Run offline JSONL/guard tests without Studio.
  -h, --help             Show this help.

After RunItem starts, keep this process open and send direct bridge requests on
stdin, for example:
  {"Screenshot":{"build_id":[M]}}
  {"WidgetTreeDump":{"build_id":[M]}}
  {"Click":{"build_id":[M],"x":100,"y":100}}
  {"TypeText":{"build_id":[M],"text":"hello native"}}
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --studio)
            studio_addr="${2:?missing value for --studio}"
            shift 2
            ;;
        --target)
            target_name="${2:?missing value for --target}"
            shift 2
            ;;
        --diag)
            target_name="makepad-example-native-text-input-diag"
            shift
            ;;
        --list-builds)
            list_only=1
            shift
            ;;
        --clear-build-id)
            clear_ids+=("${2:?missing value for --clear-build-id}")
            shift 2
            ;;
        --allow-no-clear)
            allow_no_clear=1
            shift
            ;;
        --print-jsonl)
            print_jsonl=1
            shift
            ;;
        --once)
            once=1
            shift
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

emit_jsonl() {
    printf '{"ListBuilds":[]}\n'
    if [[ "$list_only" -eq 1 ]]; then
        return
    fi
    if [[ "${#clear_ids[@]}" -gt 0 ]]; then
        for id in "${clear_ids[@]}"; do
            printf '{"ClearBuild":{"build_id":[%s]}}\n' "$id"
        done
    fi
    printf '{"RunItem":{"mount":"makepad","name":"%s"}}\n' "$target_name"
}

should_refuse_runitem() {
    [[ "$list_only" -eq 0 && "${#clear_ids[@]}" -eq 0 && "$allow_no_clear" -eq 0 ]]
}

assert_contains() {
    local text="$1"
    local pattern="$2"
    local description="$3"
    if ! printf '%s\n' "$text" | rg -q -- "$pattern"; then
        echo "self-test failed: missing ${description}: ${pattern}" >&2
        printf '%s\n' "$text" >&2
        exit 1
    fi
}

assert_not_contains() {
    local text="$1"
    local pattern="$2"
    local description="$3"
    if printf '%s\n' "$text" | rg -q -- "$pattern"; then
        echo "self-test failed: unexpected ${description}: ${pattern}" >&2
        printf '%s\n' "$text" >&2
        exit 1
    fi
}

run_self_test() {
    local output

    list_only=0
    target_name="makepad-example-native-text-input"
    clear_ids=()
    allow_no_clear=1
    output="$(emit_jsonl)"
    assert_contains "$output" '^\{"ListBuilds":\[\]\}$' "ListBuilds packet"
    assert_contains "$output" '"RunItem":\{"mount":"makepad","name":"makepad-example-native-text-input"\}' "normal RunItem packet"
    assert_not_contains "$output" '"ClearBuild"' "ClearBuild packet for allow-no-clear launch"

    list_only=0
    target_name="makepad-example-native-text-input-diag"
    clear_ids=(7)
    allow_no_clear=0
    output="$(emit_jsonl)"
    assert_contains "$output" '"ClearBuild":\{"build_id":\[7\]\}' "ClearBuild packet"
    assert_contains "$output" '"RunItem":\{"mount":"makepad","name":"makepad-example-native-text-input-diag"\}' "diagnostic RunItem packet"

    list_only=1
    target_name="makepad-example-native-text-input"
    clear_ids=()
    allow_no_clear=0
    output="$(emit_jsonl)"
    assert_contains "$output" '^\{"ListBuilds":\[\]\}$' "list-only ListBuilds packet"
    assert_not_contains "$output" '"RunItem"' "RunItem packet for list-only mode"

    list_only=0
    clear_ids=()
    allow_no_clear=0
    if ! should_refuse_runitem; then
        echo "self-test failed: unguarded RunItem was not refused" >&2
        exit 1
    fi

    list_only=0
    clear_ids=(1)
    allow_no_clear=0
    if should_refuse_runitem; then
        echo "self-test failed: guarded RunItem with ClearBuild was refused" >&2
        exit 1
    fi

    echo "NativeTextInput Studio gate self-test passed"
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

if should_refuse_runitem; then
    echo "refusing to RunItem without --clear-build-id or --allow-no-clear" >&2
    echo "run --list-builds first, then pass each old build id to --clear-build-id" >&2
    exit 2
fi

if [[ "$print_jsonl" -eq 1 ]]; then
    emit_jsonl
    exit 0
fi

bridge="target/release/cargo-makepad"
if [[ ! -x "$bridge" ]]; then
    echo "$bridge is missing or not executable; build cargo-makepad release first" >&2
    exit 1
fi

if [[ "$once" -eq 1 || ! -t 0 ]]; then
    emit_jsonl | "$bridge" studio --studio="$studio_addr"
else
    { emit_jsonl; cat; } | "$bridge" studio --studio="$studio_addr"
fi
