#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_addr="127.0.0.1:8001"
timeout=180
allow_no_clear=0
ios_device=""
android_device=""
ohos_device=""
self_test=0
dry_run=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_completion_run_all.sh [options]

Runs the full NativeTextInput completion evidence flow in a clean Studio
environment, then runs non-UI static evidence and final audit.

This script intentionally requires --allow-no-clear, then performs its own
ListBuilds check before launching any RunItem. If old NativeTextInput builds
exist, it exits before the first RunItem; clear them first or use the individual
evidence helpers.

Options:
  --studio ADDR          Studio bridge target, default 127.0.0.1:8001.
  --allow-no-clear       Required. Confirms ListBuilds showed no old builds.
  --ios-device TEXT      Required iOS simulator/device identifier.
  --android-device TEXT  Required Android device/emulator identifier.
  --ohos-device TEXT     Required OpenHarmony device identifier.
  --timeout SECONDS      Startup timeout for runtime helpers, default 180.
  --dry-run              Print the command sequence without connecting Studio.
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
        --allow-no-clear)
            allow_no_clear=1
            shift
            ;;
        --ios-device)
            ios_device="${2:?missing value for --ios-device}"
            shift 2
            ;;
        --android-device)
            android_device="${2:?missing value for --android-device}"
            shift 2
            ;;
        --ohos-device)
            ohos_device="${2:?missing value for --ohos-device}"
            shift 2
            ;;
        --timeout)
            timeout="${2:?missing value for --timeout}"
            shift 2
            ;;
        --dry-run)
            dry_run=1
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

native_textinput_build_pattern='"package"[[:space:]]*:[[:space:]]*"makepad-example-native-text-input"'

list_builds_has_native_textinput_build() {
    local output="$1"
    printf '%s\n' "$output" | rg -q "$native_textinput_build_pattern"
}

run_self_test() {
    local clean='{"Builds":{"builds":[{"build_id":[3],"mount":"makepad","package":"makepad-example-todo","active":true}]}}'
    local stale='{"Builds":{"builds":[{"build_id":[9],"mount":"makepad","package":"makepad-example-native-text-input","active":true}]}}'
    local stale_spaced='{"Builds":{"builds":[{"build_id":[10],"mount":"makepad","package" : "makepad-example-native-text-input","active":true}]}}'

    if list_builds_has_native_textinput_build "$clean"; then
        echo "self-test failed: clean Builds response matched NativeTextInput package" >&2
        exit 1
    fi
    if ! list_builds_has_native_textinput_build "$stale"; then
        echo "self-test failed: compact NativeTextInput Builds response was not detected" >&2
        exit 1
    fi
    if ! list_builds_has_native_textinput_build "$stale_spaced"; then
        echo "self-test failed: spaced NativeTextInput Builds response was not detected" >&2
        exit 1
    fi

    echo "NativeTextInput completion runner self-test passed."
}

print_dry_run() {
    local ios_label="${ios_device:-<actual iOS simulator/device name plus UDID>}"
    local android_label="${android_device:-<actual Android emulator/device name plus serial>}"
    local ohos_label="${ohos_device:-<actual OpenHarmony DevEco target plus serial>}"

    cat <<EOF
NativeTextInput completion runbook

Preflight:
  tools/native_textinput_runtime_preflight.sh --studio "$studio_addr"

Full clean run:
  tools/native_textinput_completion_run_all.sh \\
    --studio "$studio_addr" \\
    --allow-no-clear \\
    --timeout "$timeout" \\
    --ios-device "$ios_label" \\
    --android-device "$android_label" \\
    --ohos-device "$ohos_label"

Equivalent expanded sequence:
  tools/native_textinput_studio_gate.sh --studio "$studio_addr" --list-builds --once
  tools/native_textinput_macos_studio_smoke.sh --studio "$studio_addr" --allow-no-clear --timeout "$timeout"
  tools/native_textinput_macos_composition_evidence.sh --studio "$studio_addr" --timeout "$timeout" --clear-build-id <macos-smoke-build-id> --verified "shared_host clipped scroll z_order mount_queue same_frame"
  tools/native_textinput_macos_leak_smoke.sh --studio "$studio_addr" --timeout "$timeout" --clear-build-id <macos-composition-build-id>
  tools/native_textinput_device_runtime_evidence.sh --platform ios --studio "$studio_addr" --timeout "$timeout" --clear-build-id <macos-leak-build-id> --device "$ios_label" --verified "keyboard changed selection clipboard"
  tools/native_textinput_device_runtime_evidence.sh --platform android --studio "$studio_addr" --timeout "$timeout" --clear-build-id <ios-build-id> --device "$android_label" --verified "IME changed selection clipboard"
  tools/native_textinput_device_runtime_evidence.sh --platform ohos --studio "$studio_addr" --timeout "$timeout" --clear-build-id <android-build-id> --device "$ohos_label" --verified "DevEco changed selection clipboard"
  tools/native_textinput_static_evidence.sh
  tools/native_textinput_completion_audit.sh

The actual runner reads each BuildId from the evidence file produced by the
previous step and passes it as the next --clear-build-id, so every RunItem starts
from a freshly cleared Studio build.
EOF
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

if [[ "$dry_run" -eq 1 ]]; then
    print_dry_run
    exit 0
fi

if [[ "$allow_no_clear" -ne 1 ]]; then
    echo "refusing to run full completion flow without --allow-no-clear" >&2
    echo "pass --allow-no-clear only when this runner may start from a clean ListBuilds state" >&2
    exit 2
fi

if [[ -z "$ios_device" || -z "$android_device" || -z "$ohos_device" ]]; then
    echo "missing required --ios-device, --android-device, or --ohos-device" >&2
    usage >&2
    exit 2
fi

reject_placeholder_device() {
    local option="$1"
    local value="$2"
    local invalid_pattern="$3"

    if [[ "$value" =~ $invalid_pattern ]]; then
        echo "invalid --${option}-device placeholder: $value" >&2
        echo "pass the actual simulator/device name plus UDID, serial, or DevEco target identifier" >&2
        exit 2
    fi
}

reject_placeholder_device ios "$ios_device" '^(not recorded|iPhone Simulator|.*UDID recorded.*|.*UDID TBD.*|.*placeholder.*)$'
reject_placeholder_device android "$android_device" '^(not recorded|Android emulator or device|.*placeholder.*|.*serial TBD.*)$'
reject_placeholder_device ohos "$ohos_device" '^(not recorded|OpenHarmony device|.*serial or DevEco target name.*|.*target name.*|.*placeholder.*)$'

run() {
    printf '\n==> %s\n' "$*"
    "$@"
}

check_no_existing_native_textinput_builds() {
    local output
    output="$(tools/native_textinput_studio_gate.sh --studio "$studio_addr" --list-builds --once)"

    if ! printf '%s\n' "$output" | rg -q '"Builds"'; then
        echo "Studio bridge did not return a Builds response for ListBuilds" >&2
        printf '%s\n' "$output" >&2
        exit 1
    fi

    if list_builds_has_native_textinput_build "$output"; then
        echo "existing makepad-example-native-text-input build found; clear it before using --allow-no-clear" >&2
        printf '%s\n' "$output" >&2
        exit 2
    fi

    printf 'ListBuilds confirmed no existing makepad-example-native-text-input builds.\n'
}

check_no_existing_native_textinput_builds

runtime_clean_args=(--studio "$studio_addr" --allow-no-clear --timeout "$timeout")
runtime_base_args=(--studio "$studio_addr" --timeout "$timeout")

read_build_id_from_evidence() {
    local path="$1"
    if [[ ! -f "$path" ]]; then
        echo "missing evidence file: $path" >&2
        exit 1
    fi
    local build_id
    build_id="$(sed -n 's/^BuildId: //p' "$path" | head -n 1)"
    if [[ ! "$build_id" =~ ^[0-9]+$ ]]; then
        echo "could not read BuildId from $path" >&2
        exit 1
    fi
    printf '%s\n' "$build_id"
}

run tools/native_textinput_macos_studio_smoke.sh "${runtime_clean_args[@]}"

macos_smoke_build_id="$(read_build_id_from_evidence "docs/native-textinput-evidence/macos-studio-runtime.md")"

run tools/native_textinput_macos_composition_evidence.sh \
    "${runtime_base_args[@]}" \
    --clear-build-id "$macos_smoke_build_id" \
    --verified "shared_host clipped scroll z_order mount_queue same_frame"

macos_composition_build_id="$(read_build_id_from_evidence "docs/native-textinput-evidence/macos-composition-runtime.md")"

run tools/native_textinput_macos_leak_smoke.sh \
    "${runtime_base_args[@]}" \
    --clear-build-id "$macos_composition_build_id"

macos_leak_build_id="$(read_build_id_from_evidence "docs/native-textinput-evidence/macos-leak-runtime.md")"

run tools/native_textinput_device_runtime_evidence.sh \
    --platform ios \
    "${runtime_base_args[@]}" \
    --clear-build-id "$macos_leak_build_id" \
    --device "$ios_device" \
    --verified "keyboard changed selection clipboard"

ios_build_id="$(read_build_id_from_evidence "docs/native-textinput-evidence/ios-runtime.md")"

run tools/native_textinput_device_runtime_evidence.sh \
    --platform android \
    "${runtime_base_args[@]}" \
    --clear-build-id "$ios_build_id" \
    --device "$android_device" \
    --verified "IME changed selection clipboard"

android_build_id="$(read_build_id_from_evidence "docs/native-textinput-evidence/android-runtime.md")"

run tools/native_textinput_device_runtime_evidence.sh \
    --platform ohos \
    "${runtime_base_args[@]}" \
    --clear-build-id "$android_build_id" \
    --device "$ohos_device" \
    --verified "DevEco changed selection clipboard"
run tools/native_textinput_static_evidence.sh
run tools/native_textinput_completion_audit.sh
