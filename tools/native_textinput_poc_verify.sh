#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

include_blocked_targets=0
include_android_target=0
include_ohos_target=0
skip_ios_release=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_poc_verify.sh [options]

Runs the non-UI NativeTextInput POC verification gates that do not require
Studio runtime control or physical/simulator devices.

Options:
  --include-blocked-targets  Also try Android and OpenHarmony target checks.
                             These require crates.io access and the relevant
                             target/toolchain setup.
  --include-android-target   Also try the Android target check.
  --include-ohos-target      Also try the OpenHarmony target check.
  --skip-ios-release         Skip the iOS release target check.
  -h, --help                 Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --include-blocked-targets)
            include_blocked_targets=1
            include_android_target=1
            include_ohos_target=1
            shift
            ;;
        --include-android-target)
            include_android_target=1
            shift
            ;;
        --include-ohos-target)
            include_ohos_target=1
            shift
            ;;
        --skip-ios-release)
            skip_ios_release=1
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

run() {
    printf '\n==> %s\n' "$*"
    "$@"
}

native_rust_files=(
    platform/src/action.rs
    platform/src/cx.rs
    platform/src/cx_api.rs
    platform/src/lib.rs
    platform/src/native_host_schema.rs
    platform/src/native_texture_layer.rs
    platform/src/os/apple/apple_ios_native_text_input.rs
    platform/src/os/apple/apple_native_host.rs
    platform/src/os/apple/apple_native_label.rs
    platform/src/os/apple/apple_native_text_input.rs
    platform/src/os/apple/apple_webview.rs
    platform/src/os/apple/ios/ios.rs
    platform/src/os/apple/macos/macos.rs
    platform/src/os/apple/macos/macos_stdin.rs
    platform/src/os/apple/tvos/tvos.rs
    platform/src/os/headless/event_loop.rs
    platform/src/os/linux/android/android.rs
    platform/src/os/linux/android/android_jni.rs
    platform/src/os/linux/direct/linux_direct.rs
    platform/src/os/linux/open_harmony/oh_callbacks.rs
    platform/src/os/linux/open_harmony/open_harmony.rs
    platform/src/os/linux/x11/linux_x11_stdin.rs
    platform/src/os/web/web.rs
    platform/src/os/windows/windows.rs
    platform/src/os/windows/windows_stdin.rs
    widgets/src/lib.rs
    widgets/src/native_label.rs
    widgets/src/native_text_input.rs
    widgets/src/widget_tree.rs
)

run rustfmt --edition 2021 --check --config skip_children=true "${native_rust_files[@]}"
run tools/native_textinput_example_static_check.sh
run tools/native_textinput_studio_gate_static_check.sh
run tools/native_textinput_studio_gate.sh --self-test
run tools/native_textinput_runtime_preflight.sh --self-test
run tools/native_textinput_completion_run_all.sh --self-test
run tools/native_textinput_macos_studio_smoke.sh --self-test
run tools/native_textinput_macos_composition_evidence.sh --self-test
run tools/native_textinput_macos_leak_smoke.sh --self-test
run tools/native_textinput_device_runtime_evidence.sh --self-test
run tools/native_textinput_static_evidence.sh --self-test
run tools/native_textinput_completion_audit.sh --self-test
if [[ "${NATIVE_TEXTINPUT_SKIP_STAGE_REPORT:-0}" -ne 1 ]]; then
    run tools/native_textinput_completion_audit.sh --stage-report
fi
run bash -n tools/native_textinput_macos_studio_smoke.sh tools/native_textinput_macos_leak_smoke.sh tools/native_textinput_macos_composition_evidence.sh tools/native_textinput_device_runtime_evidence.sh tools/native_textinput_runtime_preflight.sh tools/native_textinput_static_evidence.sh tools/native_textinput_completion_run_all.sh tools/native_textinput_completion_audit.sh
run tools/native_textinput_host_queue_static_check.sh
run tools/native_textinput_apple_static_check.sh
run tools/native_textinput_android_static_check.sh
run tools/native_textinput_ohos_static_check.sh
run tools/native_textinput_schema_texture_static_check.sh
run cargo check -p makepad-widgets
run cargo check -p makepad-example-native-text-input --release
run cargo check -p makepad-example-native-text-input --release --features makepad-widgets/diag-native-leak-count
run cargo test -p makepad-example-native-text-input --test ui --no-run
run cargo check -p makepad-example-native-text-input --target aarch64-apple-ios

if [[ "$skip_ios_release" -eq 0 ]]; then
    run cargo check -p makepad-example-native-text-input --target aarch64-apple-ios --release
fi

run cargo test -p makepad-widgets native_ --lib
run cargo test -p makepad-platform native_ --lib
run git diff --check

if [[ "$include_android_target" -eq 1 ]]; then
    run cargo check -p makepad-widgets --target aarch64-linux-android
fi

if [[ "$include_ohos_target" -eq 1 ]]; then
    run cargo check -p makepad-platform --target aarch64-unknown-linux-ohos
fi

cat <<'EOF'

Non-UI NativeTextInput POC gates passed.

This script does not close runtime/device gates. Use Studio RunItem validation
for macOS UI behavior and leak checks, plus simulator/device validation for iOS,
Android, and OpenHarmony.
EOF
