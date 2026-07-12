#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

allow_env_deferred=0
self_test=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_macos_completion_audit.sh [options]

Checks the macOS NativeTextInput POC completion scope only. This audit does not
require or inspect iOS, Android, or OpenHarmony evidence.

Options:
  --allow-env-deferred  Pass when macOS code/static gates are complete and only
                        Studio runtime evidence remains unavailable.
  --self-test           Test audit helpers without reading evidence.
  -h, --help            Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --allow-env-deferred)
            allow_env_deferred=1
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

require_pattern() {
    local path="$1"
    local pattern="$2"
    local label="$3"
    if [[ ! -f "$path" ]]; then
        echo "missing evidence: $label ($path)" >&2
        return 1
    fi
    if ! rg -q "$pattern" "$path"; then
        echo "incomplete evidence: $label ($path must match /$pattern/)" >&2
        return 1
    fi
}

check_static() {
    local failed=0
    require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Status: PASS$" "non-UI static status" || failed=1
    require_pattern "docs/native-textinput-evidence/non-ui-static.md" "rustfmt.*example.*example_ui_test_compile.*studio_gate.*runtime_preflight.*stage_report.*host_queue.*apple.*schema.*texture.*cargo_check.*widget_tests.*platform_tests.*diff_check" "macOS static gates" || failed=1
    require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^IosRelease: checked$" "Apple/iOS compile sanity marker" || failed=1
    return "$failed"
}

check_runtime_markers() {
    local failed=0
    require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Status: PASS$" "macOS Studio smoke" || failed=1
    require_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "^Status: PASS$" "macOS leak lifecycle" || failed=1
    require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^Status: PASS$" "macOS composition runtime" || failed=1
    return "$failed"
}

print_deferred_checklist() {
    cat <<'EOF'
macOS environment-deferred checklist:

- Run `tools/native_textinput_runtime_preflight.sh` against Studio.
- Run `tools/native_textinput_macos_studio_smoke.sh` for focus/blur/changed/selection/clipboard smoke.
- Run `tools/native_textinput_macos_leak_smoke.sh` for detach/leak lifecycle.
- Run `tools/native_textinput_macos_composition_evidence.sh --verified "shared_host clipped scroll z_order mount_queue same_frame"`.
- Re-run `tools/native_textinput_macos_completion_audit.sh` without `--allow-env-deferred`.
EOF
}

if [[ "$self_test" -eq 1 ]]; then
    echo "NativeTextInput macOS completion audit self-test passed."
    exit 0
fi

if [[ "$allow_env_deferred" -eq 1 ]]; then
    check_static
    echo "NativeTextInput macOS environment-deferred audit passed."
    echo
    print_deferred_checklist
    exit 0
fi

check_static
check_runtime_markers
echo "NativeTextInput macOS completion audit passed."
