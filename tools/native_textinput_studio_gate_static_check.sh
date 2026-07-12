#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_gate="tools/native_textinput_studio_gate.sh"

require_output() {
    local output="$1"
    local pattern="$2"
    local description="$3"
    if ! printf '%s\n' "$output" | rg -q -- "$pattern"; then
        echo "missing ${description}: ${pattern}" >&2
        echo "$output" >&2
        exit 1
    fi
}

normal_jsonl="$("$studio_gate" --print-jsonl --allow-no-clear)"
require_output "$normal_jsonl" "\\{\"ListBuilds\":\\[\\]\\}" "ListBuilds packet"
require_output "$normal_jsonl" "\\{\"RunItem\":\\{\"mount\":\"makepad\",\"name\":\"makepad-example-native-text-input\"\\}\\}" "normal RunItem packet"

diag_jsonl="$("$studio_gate" --print-jsonl --diag --clear-build-id 6)"
require_output "$diag_jsonl" "\\{\"ListBuilds\":\\[\\]\\}" "diagnostic ListBuilds packet"
require_output "$diag_jsonl" "\\{\"ClearBuild\":\\{\"build_id\":\\[6\\]\\}\\}" "diagnostic ClearBuild packet"
require_output "$diag_jsonl" "\\{\"RunItem\":\\{\"mount\":\"makepad\",\"name\":\"makepad-example-native-text-input-diag\"\\}\\}" "diagnostic RunItem packet"

refusal_err="$(mktemp)"
refusal_out="$(mktemp)"
trap 'rm -f "$refusal_err" "$refusal_out"' EXIT

if "$studio_gate" --print-jsonl >"$refusal_out" 2>"$refusal_err"; then
    echo "Studio gate accepted RunItem without clear-build guard" >&2
    exit 1
fi
require_output "$(cat "$refusal_err")" "refusing to RunItem without --clear-build-id or --allow-no-clear" "clear-build refusal"

echo "NativeTextInput Studio gate static contract check passed."
