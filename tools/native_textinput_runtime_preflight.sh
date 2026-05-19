#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_addr="127.0.0.1:8001"
self_test=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_runtime_preflight.sh [options]

Checks the runtime evidence environment without launching any UI RunItem.
This verifies the Studio bridge can answer ListBuilds and that the NativeTextInput
runtime helpers are executable before starting the full evidence flow.

Options:
  --studio ADDR   Studio bridge target, default 127.0.0.1:8001.
  --self-test     Run offline helper/JSONL tests without Studio.
  -h, --help      Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --studio)
            studio_addr="${2:?missing value for --studio}"
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

helpers=(
    tools/native_textinput_studio_gate.sh
    tools/native_textinput_macos_studio_smoke.sh
    tools/native_textinput_macos_leak_smoke.sh
    tools/native_textinput_macos_composition_evidence.sh
    tools/native_textinput_device_runtime_evidence.sh
    tools/native_textinput_static_evidence.sh
    tools/native_textinput_completion_run_all.sh
    tools/native_textinput_completion_audit.sh
)

check_helpers() {
    local helper
    for helper in "${helpers[@]}"; do
    if [[ ! -x "$helper" ]]; then
        echo "$helper is missing or not executable" >&2
        exit 1
    fi
    done
}

check_gate_jsonl() {
    local normal_jsonl diag_jsonl list_jsonl

    normal_jsonl="$(tools/native_textinput_studio_gate.sh --print-jsonl --allow-no-clear)"
    diag_jsonl="$(tools/native_textinput_studio_gate.sh --print-jsonl --diag --clear-build-id 1)"
    list_jsonl="$(tools/native_textinput_studio_gate.sh --print-jsonl --list-builds)"

    if ! printf '%s\n' "$normal_jsonl" | rg -q '"RunItem":\{"mount":"makepad","name":"makepad-example-native-text-input"\}'; then
        echo "normal NativeTextInput RunItem JSONL is not registered as expected" >&2
        exit 1
    fi
    if ! printf '%s\n' "$diag_jsonl" | rg -q '"ClearBuild":\{"build_id":\[1\]\}'; then
        echo "diagnostic NativeTextInput JSONL does not clear the previous build" >&2
        exit 1
    fi
    if ! printf '%s\n' "$diag_jsonl" | rg -q '"RunItem":\{"mount":"makepad","name":"makepad-example-native-text-input-diag"\}'; then
        echo "diagnostic NativeTextInput RunItem JSONL is not registered as expected" >&2
        exit 1
    fi
    if printf '%s\n' "$list_jsonl" | rg -q '"RunItem"'; then
        echo "list-builds JSONL unexpectedly contains RunItem" >&2
        exit 1
    fi
}

run_self_test() {
    check_helpers
    tools/native_textinput_studio_gate.sh --self-test
    check_gate_jsonl
    echo "NativeTextInput runtime preflight self-test passed"
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

bridge="target/release/cargo-makepad"
if [[ ! -x "$bridge" ]]; then
    echo "$bridge is missing or not executable; build cargo-makepad release first" >&2
    exit 1
fi

check_helpers
check_gate_jsonl

list_builds_output="$(tools/native_textinput_studio_gate.sh --studio "$studio_addr" --list-builds --once)"
if ! printf '%s\n' "$list_builds_output" | rg -q '"Builds"'; then
    echo "Studio bridge did not return a Builds response for ListBuilds" >&2
    printf '%s\n' "$list_builds_output" >&2
    exit 1
fi

cat <<EOF
NativeTextInput runtime preflight passed.

Studio: $studio_addr
ListBuilds:
$list_builds_output
EOF
