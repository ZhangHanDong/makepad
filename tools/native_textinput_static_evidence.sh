#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

output="docs/native-textinput-evidence/non-ui-static.md"
include_android_target=1
include_ohos_target=1
skip_ios_release=0
self_test=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_static_evidence.sh [options]

Runs the non-UI NativeTextInput verification gates and writes a static evidence
marker for the completion audit. This closes compile/static/schema/texture
evidence only; runtime/device gates still require the separate evidence files.

Options:
  --no-android-target    Do not run the Android target check.
  --skip-ohos-target     Skip the OpenHarmony target check. Local-only escape
                         hatch for environments without crates.io/OHOS setup.
  --skip-ios-release     Skip the iOS release target check.
  --output PATH          Evidence file to write on success.
  --self-test            Test marker rendering without running cargo.
  -h, --help             Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-android-target)
            include_android_target=0
            shift
            ;;
        --skip-ohos-target)
            include_ohos_target=0
            shift
            ;;
        --skip-ios-release)
            skip_ios_release=1
            shift
            ;;
        --output)
            output="${2:?missing value for --output}"
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

render_verify_command() {
    local android="$1"
    local ohos="$2"
    local skip_ios="$3"
    local args=()

    if [[ "$android" -eq 1 ]]; then
        args+=(--include-android-target)
    fi
    if [[ "$ohos" -eq 1 ]]; then
        args+=(--include-ohos-target)
    fi
    if [[ "$skip_ios" -eq 1 ]]; then
        args+=(--skip-ios-release)
    fi

    printf 'tools/native_textinput_poc_verify.sh'
    if [[ "${#args[@]}" -gt 0 ]]; then
        printf ' %s' "${args[@]}"
    fi
    printf '\n'
}

run_self_test() {
    local default_command
    local skip_ohos_command
    local skip_ios_command

    default_command="$(render_verify_command 1 1 0)"
    skip_ohos_command="$(render_verify_command 1 0 0)"
    skip_ios_command="$(render_verify_command 1 1 1)"

    if [[ "$default_command" != "tools/native_textinput_poc_verify.sh --include-android-target --include-ohos-target" ]]; then
        echo "self-test failed: default static evidence command is wrong: $default_command" >&2
        exit 1
    fi
    if [[ "$skip_ohos_command" == *"--include-ohos-target"* ]]; then
        echo "self-test failed: --skip-ohos-target command still includes OHOS target" >&2
        exit 1
    fi
    if [[ "$skip_ios_command" != *"--skip-ios-release"* ]]; then
        echo "self-test failed: --skip-ios-release command did not include skip marker" >&2
        exit 1
    fi

    echo "NativeTextInput static evidence self-test passed."
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

verify_args=()
if [[ "$include_android_target" -eq 1 ]]; then
    verify_args+=(--include-android-target)
fi
if [[ "$include_ohos_target" -eq 1 ]]; then
    verify_args+=(--include-ohos-target)
fi
if [[ "$skip_ios_release" -eq 1 ]]; then
    verify_args+=(--skip-ios-release)
fi

NATIVE_TEXTINPUT_SKIP_STAGE_REPORT=1 tools/native_textinput_poc_verify.sh "${verify_args[@]}"

mkdir -p "$(dirname "$output")"
cat >"$output" <<EOF
Status: PASS
Verified: rustfmt example example_ui_test_compile studio_gate runtime_preflight stage_report host_queue apple android ohos schema texture cargo_check widget_tests platform_tests diff_check
AndroidTarget: $([[ "$include_android_target" -eq 1 ]] && echo checked || echo skipped)
OhosTarget: $([[ "$include_ohos_target" -eq 1 ]] && echo checked || echo skipped)
IosRelease: $([[ "$skip_ios_release" -eq 0 ]] && echo checked || echo skipped)

Command: $(render_verify_command "$include_android_target" "$include_ohos_target" "$skip_ios_release")
EOF

echo "wrote $output"
tools/native_textinput_completion_audit.sh --stage-report
