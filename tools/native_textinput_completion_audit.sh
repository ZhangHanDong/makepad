#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

run_non_ui=0
skip_ios_release=0
skip_ohos_target=0
self_test=0
stage_report=0
allow_env_deferred=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_completion_audit.sh [options]

Checks whether the NativeTextInput roadmap has real runtime/device evidence.
This is intentionally separate from tools/native_textinput_poc_verify.sh:
cargo/static/unit gates do not close Studio, simulator, DevEco, or device gates.

Options:
  --run-non-ui              Run tools/native_textinput_static_evidence.sh first.
  --include-blocked-targets Compatibility alias; required target checks are
                             already included by native_textinput_static_evidence.sh.
  --skip-ios-release        Local-only escape hatch forwarded to
                             native_textinput_static_evidence.sh. The final
                             audit will still fail while IosRelease is skipped.
  --skip-ohos-target        Local-only escape hatch forwarded to
                             native_textinput_static_evidence.sh. The final
                             audit will still fail while OhosTarget is skipped.
  --stage-report            Print a stage-to-evidence status report and exit.
  --allow-env-deferred      Pass when code/static gates are complete and only
                             Studio/simulator/device/OHOS environment evidence
                             remains. Prints the deferred verification checklist.
  --self-test               Test audit guard helpers without reading evidence.
  -h, --help                Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --run-non-ui)
            run_non_ui=1
            shift
            ;;
        --include-blocked-targets)
            # Compatibility alias retained for older runbooks. Static evidence
            # already includes the Android and OpenHarmony target checks by default.
            shift
            ;;
        --skip-ios-release)
            skip_ios_release=1
            shift
            ;;
        --skip-ohos-target)
            skip_ohos_target=1
            shift
            ;;
        --stage-report)
            stage_report=1
            shift
            ;;
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

device_placeholder_pattern() {
    case "$1" in
        ios)
            printf '%s\n' '^(not recorded|iPhone Simulator|.*UDID recorded.*|.*UDID TBD.*|.*placeholder.*)$'
            ;;
        android)
            printf '%s\n' '^(not recorded|Android emulator or device|.*placeholder.*|.*serial TBD.*)$'
            ;;
        ohos)
            printf '%s\n' '^(not recorded|OpenHarmony device|.*serial or DevEco target name.*|.*target name.*|.*placeholder.*|.*127[.]0[.]0[.]1:[0-9]+.*|.*localhost.*)$'
            ;;
        *)
            return 1
            ;;
    esac
}

is_placeholder_device() {
    local platform="$1"
    local value="$2"
    local pattern
    pattern="$(device_placeholder_pattern "$platform")"
    [[ "$value" =~ $pattern ]]
}

run_self_test() {
    if is_placeholder_device ios "iPhone 16 Simulator (UDID 00008110-001234D20E91801E)"; then
        echo "self-test failed: concrete iOS simulator was treated as placeholder" >&2
        exit 1
    fi
    if ! is_placeholder_device ios "iPhone Simulator"; then
        echo "self-test failed: generic iOS simulator was not rejected" >&2
        exit 1
    fi
    if ! is_placeholder_device ios "iPhone 16 Simulator (UDID recorded)"; then
        echo "self-test failed: copied iOS README placeholder was not rejected" >&2
        exit 1
    fi
    if is_placeholder_device android "Pixel 8 API 35 emulator (serial emulator-5554)"; then
        echo "self-test failed: concrete Android emulator was treated as placeholder" >&2
        exit 1
    fi
    if ! is_placeholder_device android "Android emulator or device"; then
        echo "self-test failed: generic Android device was not rejected" >&2
        exit 1
    fi
    if is_placeholder_device ohos "OpenHarmony RK3568 DevEco target (serial OHOS-123456)"; then
        echo "self-test failed: concrete OpenHarmony device was treated as placeholder" >&2
        exit 1
    fi
    if ! is_placeholder_device ohos "OpenHarmony device"; then
        echo "self-test failed: generic OpenHarmony device was not rejected" >&2
        exit 1
    fi
    if ! is_placeholder_device ohos "OpenHarmony local simulator (127.0.0.1:5555)"; then
        echo "self-test failed: OpenHarmony loopback simulator was not rejected" >&2
        exit 1
    fi

    echo "NativeTextInput completion audit self-test passed."
}

evidence_status() {
    local path="$1"
    shift

    if [[ ! -f "$path" ]]; then
        printf 'missing %s' "$path"
        return
    fi

    local pattern
    for pattern in "$@"; do
        if ! rg -q "$pattern" "$path"; then
            printf 'incomplete %s' "$path"
            return
        fi
    done

    printf 'present %s' "$path"
}

combine_stage_status() {
    local status="ready"
    local item
    for item in "$@"; do
        if [[ -z "$item" ]]; then
            continue
        fi
        if [[ "$item" == missing* ]]; then
            printf 'blocked'
            return
        fi
        if [[ "$item" == incomplete* ]]; then
            status="blocked"
        fi
    done
    printf '%s' "$status"
}

print_stage_report_line() {
    local stage="$1"
    local deliverable="$2"
    shift 2
    local status
    status="$(combine_stage_status "$@")"
    printf '%-5s %-8s %s\n' "$stage" "$status" "$deliverable"
    local item
    for item in "$@"; do
        if [[ -z "$item" ]]; then
            continue
        fi
        printf '              - %s\n' "$item"
    done
}

marker_file_status() {
    local path="$1"
    local marker="$2"
    local label="$3"
    local value

    if [[ ! -f "$path" ]]; then
        return
    fi

    value="$(sed -n "s/^${marker}: //p" "$path" | head -n 1)"
    if [[ -z "$value" ]]; then
        printf 'incomplete %s missing %s marker' "$path" "$marker"
        return
    fi
    if [[ "$value" == /path/to/* ]]; then
        printf 'incomplete %s has placeholder %s marker' "$path" "$marker"
        return
    fi
    if [[ ! -f "$value" || ! -r "$value" || ! -s "$value" ]]; then
        printf 'incomplete %s has unreadable or empty %s file' "$path" "$marker"
        return
    fi

    printf 'present %s' "$label"
}

marker_file_pattern_status() {
    local path="$1"
    local marker="$2"
    local pattern="$3"
    local label="$4"
    local value

    if [[ ! -f "$path" ]]; then
        return
    fi

    value="$(sed -n "s/^${marker}: //p" "$path" | head -n 1)"
    if [[ -z "$value" || "$value" == /path/to/* || ! -f "$value" || ! -r "$value" || ! -s "$value" ]]; then
        return
    fi
    if ! rg -q "$pattern" "$value"; then
        printf 'incomplete %s %s content does not match required Studio response' "$path" "$marker"
        return
    fi

    printf 'present %s' "$label"
}

device_marker_status() {
    local path="$1"
    local platform="$2"
    local label="$3"
    local value

    if [[ ! -f "$path" ]]; then
        return
    fi

    value="$(sed -n "s/^Device: //p" "$path" | head -n 1)"
    if [[ -z "$value" ]]; then
        printf 'incomplete %s missing Device marker' "$path"
        return
    fi
    if is_placeholder_device "$platform" "$value"; then
        printf 'incomplete %s has placeholder Device marker' "$path"
        return
    fi

    printf 'present %s' "$label"
}

print_stage_report() {
    local non_ui_basic non_ui_final non_ui_ios non_ui_android non_ui_ohos non_ui_schema non_ui_texture
    local macos_smoke macos_leak macos_composition ios android ohos
    local macos_smoke_screenshot macos_smoke_clipped macos_smoke_transcript macos_smoke_transcript_content
    local macos_leak_transcript macos_leak_transcript_content
    local macos_composition_screenshot macos_composition_transcript macos_composition_transcript_screenshot
    local macos_composition_transcript_clipped macos_composition_transcript_button
    local ios_device ios_transcript ios_transcript_content
    local android_device android_transcript android_transcript_content
    local ohos_device ohos_transcript ohos_transcript_content
    non_ui_basic="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^Verified: .*rustfmt.*example.*example_ui_test_compile.*studio_gate.*runtime_preflight.*stage_report.*host_queue.*apple.*android.*ohos.*schema.*texture.*cargo_check.*widget_tests.*platform_tests.*diff_check")"
    non_ui_final="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^Verified: .*rustfmt.*example.*example_ui_test_compile.*studio_gate.*runtime_preflight.*stage_report.*host_queue.*apple.*android.*ohos.*schema.*texture.*cargo_check.*widget_tests.*platform_tests.*diff_check" \
        "^AndroidTarget: checked$" \
        "^OhosTarget: checked$" \
        "^IosRelease: checked$")"
    non_ui_ios="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^IosRelease: checked$")"
    non_ui_android="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^AndroidTarget: checked$")"
    non_ui_ohos="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^OhosTarget: checked$")"
    non_ui_schema="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^Verified: .*schema")"
    non_ui_texture="$(evidence_status \
        "docs/native-textinput-evidence/non-ui-static.md" \
        "^Status: PASS$" \
        "^Verified: .*texture")"
    macos_smoke="$(evidence_status \
        "docs/native-textinput-evidence/macos-studio-runtime.md" \
        "^Status: PASS$" \
        "^RunItem: makepad-example-native-text-input$" \
        "^Verified: .*focus.*blur.*changed.*selection.*clipboard" \
        "^BuildId: [0-9]+$" \
        "^Screenshot: .+" \
        "^ClippedScreenshot: .+" \
        "^Transcript: .+")"
    macos_leak="$(evidence_status \
        "docs/native-textinput-evidence/macos-leak-runtime.md" \
        "^Status: PASS$" \
        "^Verified: .*ClearBuild.*RunItem.*detach.*leak" \
        "^BuildId: [0-9]+$" \
        "^Transcript: .+")"
    macos_composition="$(evidence_status \
        "docs/native-textinput-evidence/macos-composition-runtime.md" \
        "^Status: PASS$" \
        "^RunItem: makepad-example-native-text-input$" \
        "^Verified: .*shared_host.*clipped.*scroll.*z_order.*mount_queue.*same_frame" \
        "^BuildId: [0-9]+$" \
        "^AfterScreenshot: .+" \
        "^Transcript: .+")"
    ios="$(evidence_status \
        "docs/native-textinput-evidence/ios-runtime.md" \
        "^Status: PASS$" \
        "^RunItem: makepad-example-native-text-input-ios-sim$" \
        "^Verified: .*keyboard.*changed.*selection.*clipboard" \
        "^BuildId: [0-9]+$" \
        "^Device: .+" \
        "^Transcript: .+")"
    android="$(evidence_status \
        "docs/native-textinput-evidence/android-runtime.md" \
        "^Status: PASS$" \
        "^RunItem: makepad-example-native-text-input-android$" \
        "^Verified: .*IME.*changed.*selection.*clipboard" \
        "^BuildId: [0-9]+$" \
        "^Device: .+" \
        "^Transcript: .+")"
    ohos="$(evidence_status \
        "docs/native-textinput-evidence/ohos-runtime.md" \
        "^Status: PASS$" \
        "^RunItem: makepad-example-native-text-input-ohos$" \
        "^Verified: .*DevEco.*changed.*selection.*clipboard" \
        "^BuildId: [0-9]+$" \
        "^Device: .+" \
        "^Transcript: .+")"
    macos_smoke_screenshot="$(marker_file_status "docs/native-textinput-evidence/macos-studio-runtime.md" "Screenshot" "macOS smoke screenshot file")"
    macos_smoke_clipped="$(marker_file_status "docs/native-textinput-evidence/macos-studio-runtime.md" "ClippedScreenshot" "macOS smoke clipped screenshot file")"
    macos_smoke_transcript="$(marker_file_status "docs/native-textinput-evidence/macos-studio-runtime.md" "Transcript" "macOS smoke transcript file")"
    macos_smoke_transcript_content="$(marker_file_pattern_status "docs/native-textinput-evidence/macos-studio-runtime.md" "Transcript" '"Screenshot"|WidgetQuery|WidgetTreeDump' "macOS smoke transcript content")"
    macos_leak_transcript="$(marker_file_status "docs/native-textinput-evidence/macos-leak-runtime.md" "Transcript" "macOS leak transcript file")"
    macos_leak_transcript_content="$(marker_file_pattern_status "docs/native-textinput-evidence/macos-leak-runtime.md" "Transcript" 'QueryLogResults.*NativeTextInput live count' "macOS leak transcript content")"
    macos_composition_screenshot="$(marker_file_status "docs/native-textinput-evidence/macos-composition-runtime.md" "AfterScreenshot" "macOS composition after screenshot file")"
    macos_composition_transcript="$(marker_file_status "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" "macOS composition transcript file")"
    macos_composition_transcript_screenshot="$(marker_file_pattern_status "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" '"Screenshot"' "macOS composition transcript screenshot content")"
    macos_composition_transcript_clipped="$(marker_file_pattern_status "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" 'WidgetQuery.*clipped_native_input' "macOS composition clipped input transcript content")"
    macos_composition_transcript_button="$(marker_file_pattern_status "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" 'WidgetQuery.*set_both_button' "macOS composition same-frame button transcript content")"
    ios_device="$(device_marker_status "docs/native-textinput-evidence/ios-runtime.md" ios "iOS device marker")"
    ios_transcript="$(marker_file_status "docs/native-textinput-evidence/ios-runtime.md" "Transcript" "iOS transcript file")"
    ios_transcript_content="$(marker_file_pattern_status "docs/native-textinput-evidence/ios-runtime.md" "Transcript" 'AppStarted|RunViewCreated' "iOS transcript content")"
    android_device="$(device_marker_status "docs/native-textinput-evidence/android-runtime.md" android "Android device marker")"
    android_transcript="$(marker_file_status "docs/native-textinput-evidence/android-runtime.md" "Transcript" "Android transcript file")"
    android_transcript_content="$(marker_file_pattern_status "docs/native-textinput-evidence/android-runtime.md" "Transcript" 'AppStarted|RunViewCreated' "Android transcript content")"
    ohos_device="$(device_marker_status "docs/native-textinput-evidence/ohos-runtime.md" ohos "OpenHarmony device marker")"
    ohos_transcript="$(marker_file_status "docs/native-textinput-evidence/ohos-runtime.md" "Transcript" "OpenHarmony transcript file")"
    ohos_transcript_content="$(marker_file_pattern_status "docs/native-textinput-evidence/ohos-runtime.md" "Transcript" 'AppStarted|RunViewCreated' "OpenHarmony transcript content")"

    cat <<'EOF'
NativeTextInput stage evidence report

Status values:
  ready   all required marker files for this stage are present
  blocked at least one required marker file is missing or incomplete

EOF
    print_stage_report_line "P0A" "compile/static skeleton" "$non_ui_basic"
    print_stage_report_line "P0B" "macOS runtime loop and lifecycle" "$non_ui_basic" "$macos_smoke" "$macos_smoke_screenshot" "$macos_smoke_clipped" "$macos_smoke_transcript" "$macos_smoke_transcript_content" "$macos_leak" "$macos_leak_transcript" "$macos_leak_transcript_content"
    print_stage_report_line "P1" "shared typed native host registry" "$non_ui_basic" "$macos_composition" "$macos_composition_screenshot" "$macos_composition_transcript" "$macos_composition_transcript_screenshot" "$macos_composition_transcript_clipped" "$macos_composition_transcript_button"
    print_stage_report_line "P1.5" "clipped native host composition" "$non_ui_basic" "$macos_composition" "$macos_composition_screenshot" "$macos_composition_transcript" "$macos_composition_transcript_screenshot" "$macos_composition_transcript_clipped" "$macos_composition_transcript_button"
    print_stage_report_line "P2" "iOS UITextField runtime" "$non_ui_ios" "$ios" "$ios_device" "$ios_transcript" "$ios_transcript_content"
    print_stage_report_line "P3" "Android EditText runtime" "$non_ui_android" "$android" "$android_device" "$android_transcript" "$android_transcript_content"
    print_stage_report_line "P4" "NativeMountQueue same-frame runtime" "$non_ui_basic" "$macos_composition" "$macos_composition_screenshot" "$macos_composition_transcript" "$macos_composition_transcript_screenshot" "$macos_composition_transcript_clipped" "$macos_composition_transcript_button"
    print_stage_report_line "P5" "OpenHarmony ArkUI runtime" "$non_ui_ohos" "$ohos" "$ohos_device" "$ohos_transcript" "$ohos_transcript_content"
    print_stage_report_line "P6" "schema/codegen seed" "$non_ui_schema"
    print_stage_report_line "P7" "native texture layer seed" "$non_ui_texture"
    printf '\nFinal completion gate: %s\n' "$(combine_stage_status "$non_ui_final" "$macos_smoke" "$macos_smoke_screenshot" "$macos_smoke_clipped" "$macos_smoke_transcript" "$macos_smoke_transcript_content" "$macos_leak" "$macos_leak_transcript" "$macos_leak_transcript_content" "$macos_composition" "$macos_composition_screenshot" "$macos_composition_transcript" "$macos_composition_transcript_screenshot" "$macos_composition_transcript_clipped" "$macos_composition_transcript_button" "$ios" "$ios_device" "$ios_transcript" "$ios_transcript_content" "$android" "$android_device" "$android_transcript" "$android_transcript_content" "$ohos" "$ohos_device" "$ohos_transcript" "$ohos_transcript_content")"
}

print_env_deferred_checklist() {
    cat <<'EOF'
Environment-deferred verification checklist:

- Run `tools/native_textinput_runtime_preflight.sh` against Studio.
- Run `tools/native_textinput_completion_run_all.sh --dry-run ...` to confirm the exact sequence.
- Run `tools/native_textinput_completion_run_all.sh --allow-no-clear ...` in a clean Studio/device environment.
- Regenerate `docs/native-textinput-evidence/non-ui-static.md` without `--skip-ohos-target`.
- Capture `docs/native-textinput-evidence/macos-studio-runtime.md`.
- Capture `docs/native-textinput-evidence/macos-leak-runtime.md`.
- Capture `docs/native-textinput-evidence/macos-composition-runtime.md`.
- Capture `docs/native-textinput-evidence/ios-runtime.md`.
- Capture `docs/native-textinput-evidence/android-runtime.md`.
- Capture `docs/native-textinput-evidence/ohos-runtime.md`.
- Re-run `tools/native_textinput_completion_audit.sh` without `--allow-env-deferred`.
EOF
}

run_env_deferred_audit() {
    local failed=0

    require_env_deferred_pattern() {
        local path="$1"
        local pattern="$2"
        local label="$3"
        if [[ ! -f "$path" ]]; then
            echo "missing code/static evidence: $label ($path)" >&2
            failed=1
            return
        fi
        if ! rg -q "$pattern" "$path"; then
            echo "incomplete code/static evidence: $label ($path must match /$pattern/)" >&2
            failed=1
        fi
    }

    require_env_deferred_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Status: PASS$" "non-UI static status"
    require_env_deferred_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Verified: .*rustfmt.*example.*example_ui_test_compile.*studio_gate.*runtime_preflight.*stage_report.*host_queue.*apple.*android.*ohos.*schema.*texture.*cargo_check.*widget_tests.*platform_tests.*diff_check" "non-UI static gates"
    require_env_deferred_pattern "docs/native-textinput-evidence/non-ui-static.md" "^AndroidTarget: checked$" "Android target compile/static check"
    require_env_deferred_pattern "docs/native-textinput-evidence/non-ui-static.md" "^IosRelease: checked$" "iOS release compile/static check"
    require_env_deferred_pattern "docs/native-textinput-evidence/non-ui-static.md" "^OhosTarget: (checked|skipped)$" "OpenHarmony target marker"
    require_env_deferred_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Command: tools/native_textinput_poc_verify.sh" "non-UI verifier command"

    if [[ "$failed" -ne 0 ]]; then
        echo "NativeTextInput environment-deferred audit failed." >&2
        return 1
    fi

    echo "NativeTextInput environment-deferred audit passed."
    echo
    print_stage_report
    echo
    print_env_deferred_checklist
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

if [[ "$run_non_ui" -eq 1 ]]; then
    evidence_args=()
    if [[ "$skip_ios_release" -eq 1 ]]; then
        evidence_args+=(--skip-ios-release)
    fi
    if [[ "$skip_ohos_target" -eq 1 ]]; then
        evidence_args+=(--skip-ohos-target)
    fi
    tools/native_textinput_static_evidence.sh "${evidence_args[@]}"
fi

if [[ "$stage_report" -eq 1 ]]; then
    print_stage_report
    exit 0
fi

if [[ "$allow_env_deferred" -eq 1 ]]; then
    run_env_deferred_audit
    exit $?
fi

missing=0

require_file() {
    local path="$1"
    local label="$2"

    if [[ ! -f "$path" ]]; then
        echo "missing evidence: $label ($path)" >&2
        missing=1
        return
    fi
}

require_pattern() {
    local path="$1"
    local pattern="$2"
    local label="$3"

    if [[ ! -f "$path" ]]; then
        return
    fi

    if ! rg -q "$pattern" "$path"; then
        echo "incomplete evidence: $label ($path must match /$pattern/)" >&2
        missing=1
    fi
}

require_not_pattern() {
    local path="$1"
    local pattern="$2"
    local label="$3"

    if [[ ! -f "$path" ]]; then
        return
    fi

    if rg -q "$pattern" "$path"; then
        echo "invalid evidence: $label ($path must not match /$pattern/)" >&2
        missing=1
    fi
}

require_marker_file() {
    local path="$1"
    local marker="$2"
    local label="$3"
    local value

    if [[ ! -f "$path" ]]; then
        return
    fi

    value="$(sed -n "s/^${marker}: //p" "$path" | head -n 1)"
    if [[ -z "$value" ]]; then
        return
    fi

    if [[ "$value" == /path/to/* ]]; then
        echo "invalid evidence: $label ($path has placeholder ${marker}: $value)" >&2
        missing=1
        return
    fi

    if [[ ! -f "$value" || ! -r "$value" || ! -s "$value" ]]; then
        echo "invalid evidence: $label ($path has ${marker}: $value, but that file is not a readable non-empty file)" >&2
        missing=1
    fi
}

require_marker_file_pattern() {
    local path="$1"
    local marker="$2"
    local pattern="$3"
    local label="$4"
    local value

    if [[ ! -f "$path" ]]; then
        return
    fi

    value="$(sed -n "s/^${marker}: //p" "$path" | head -n 1)"
    if [[ -z "$value" || "$value" == /path/to/* || ! -f "$value" || ! -r "$value" || ! -s "$value" ]]; then
        return
    fi

    if ! rg -q "$pattern" "$value"; then
        echo "invalid evidence: $label ($path has ${marker}: $value, but it does not match /$pattern/)" >&2
        missing=1
    fi
}

require_device_marker() {
    local path="$1"
    local label="$2"
    local platform="$3"
    local value

    if [[ ! -f "$path" ]]; then
        return
    fi

    value="$(sed -n "s/^Device: //p" "$path" | head -n 1)"
    if [[ -z "$value" ]]; then
        echo "incomplete evidence: $label ($path must include Device: with a real target identifier)" >&2
        missing=1
        return
    fi

    if is_placeholder_device "$platform" "$value"; then
        echo "invalid evidence: $label ($path has placeholder Device: $value)" >&2
        missing=1
    fi
}

require_file "docs/native-textinput-evidence/non-ui-static.md" "non-UI compile/static/schema evidence"
require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Status: PASS$" "non-UI static status"
require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Verified: .*rustfmt.*example.*example_ui_test_compile.*studio_gate.*runtime_preflight.*stage_report.*host_queue.*apple.*android.*ohos.*schema.*texture.*cargo_check.*widget_tests.*platform_tests.*diff_check" "non-UI static gates"
require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^AndroidTarget: checked$" "Android target check"
require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^OhosTarget: checked$" "OpenHarmony target check"
require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^IosRelease: checked$" "iOS release target check"
require_pattern "docs/native-textinput-evidence/non-ui-static.md" "^Command: tools/native_textinput_poc_verify.sh .*--include-android-target.*--include-ohos-target" "non-UI static command"

require_file "docs/native-textinput-evidence/macos-studio-runtime.md" "macOS Studio RunItem smoke"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Status: PASS$" "macOS Studio status"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^RunItem: makepad-example-native-text-input$" "macOS normal RunItem"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Example: examples/native_text_input$" "macOS example path"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Verified: .*focus.*blur.*changed.*selection.*clipboard" "macOS runtime behavior"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^BuildId: [0-9]+$" "macOS Studio build id"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Screenshot: .+" "macOS Studio screenshot path"
require_marker_file "docs/native-textinput-evidence/macos-studio-runtime.md" "Screenshot" "macOS Studio screenshot file"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^ClippedScreenshot: .+" "macOS Studio clipped screenshot path"
require_marker_file "docs/native-textinput-evidence/macos-studio-runtime.md" "ClippedScreenshot" "macOS Studio clipped screenshot file"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Transcript: .+" "macOS Studio transcript path"
require_marker_file "docs/native-textinput-evidence/macos-studio-runtime.md" "Transcript" "macOS Studio transcript file"
require_marker_file_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "Transcript" '"Screenshot"|WidgetQuery|WidgetTreeDump' "macOS Studio transcript content"
require_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "^Command: .*native_textinput_macos_studio_smoke\\.sh" "macOS Studio command"
require_not_pattern "docs/native-textinput-evidence/macos-studio-runtime.md" "cargo test .*--test ui|makepad_test" "macOS Studio runtime evidence must use Studio RunItem helpers"

require_file "docs/native-textinput-evidence/macos-leak-runtime.md" "macOS detach/leak lifecycle"
require_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "^Status: PASS$" "macOS leak status"
require_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "^Verified: .*ClearBuild.*RunItem.*detach.*leak" "macOS leak behavior"
require_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "^BuildId: [0-9]+$" "macOS leak build id"
require_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "^Transcript: .+" "macOS leak transcript path"
require_marker_file "docs/native-textinput-evidence/macos-leak-runtime.md" "Transcript" "macOS leak transcript file"
require_marker_file_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "Transcript" 'QueryLogResults.*NativeTextInput live count' "macOS leak transcript content"
require_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "^Command: .*native_textinput_macos_leak_smoke\\.sh" "macOS leak command"
require_not_pattern "docs/native-textinput-evidence/macos-leak-runtime.md" "cargo test .*--test ui|makepad_test" "macOS leak evidence must use Studio RunItem helpers"

require_file "docs/native-textinput-evidence/macos-composition-runtime.md" "macOS P1/P1.5/P4 composition runtime"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^Status: PASS$" "macOS composition status"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^RunItem: makepad-example-native-text-input$" "macOS composition RunItem"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^Example: examples/native_text_input$" "macOS composition example path"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^Verified: .*shared_host.*clipped.*scroll.*z_order.*mount_queue.*same_frame" "macOS composition behavior"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^BuildId: [0-9]+$" "macOS composition build id"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^AfterScreenshot: .+" "macOS composition after screenshot path"
require_marker_file "docs/native-textinput-evidence/macos-composition-runtime.md" "AfterScreenshot" "macOS composition after screenshot file"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^Transcript: .+" "macOS composition transcript path"
require_marker_file "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" "macOS composition transcript file"
require_marker_file_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" '"Screenshot"' "macOS composition transcript screenshot content"
require_marker_file_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" 'WidgetQuery.*clipped_native_input' "macOS composition transcript clipped input content"
require_marker_file_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "Transcript" 'WidgetQuery.*set_both_button' "macOS composition transcript same-frame button content"
require_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "^Command: .*native_textinput_macos_composition_evidence\\.sh" "macOS composition command"
require_not_pattern "docs/native-textinput-evidence/macos-composition-runtime.md" "cargo test .*--test ui|makepad_test" "macOS composition evidence must use Studio RunItem helpers"

require_file "docs/native-textinput-evidence/ios-runtime.md" "iOS simulator/device runtime"
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^Status: PASS$" "iOS status"
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^RunItem: makepad-example-native-text-input-ios-sim$" "iOS RunItem"
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^Example: examples/native_text_input$" "iOS example path"
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^Verified: .*keyboard.*changed.*selection.*clipboard" "iOS runtime behavior"
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^BuildId: [0-9]+$" "iOS build id"
require_device_marker "docs/native-textinput-evidence/ios-runtime.md" "iOS device" ios
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^Transcript: .+" "iOS transcript path"
require_marker_file "docs/native-textinput-evidence/ios-runtime.md" "Transcript" "iOS transcript file"
require_marker_file_pattern "docs/native-textinput-evidence/ios-runtime.md" "Transcript" 'AppStarted|RunViewCreated' "iOS transcript content"
require_pattern "docs/native-textinput-evidence/ios-runtime.md" "^Command: .*native_textinput_device_runtime_evidence\\.sh" "iOS evidence command"
require_not_pattern "docs/native-textinput-evidence/ios-runtime.md" "cargo test .*--test ui|makepad_test" "iOS evidence must use simulator/device helpers"

require_file "docs/native-textinput-evidence/android-runtime.md" "Android device/emulator runtime"
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^Status: PASS$" "Android status"
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^RunItem: makepad-example-native-text-input-android$" "Android RunItem"
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^Example: examples/native_text_input$" "Android example path"
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^Verified: .*IME.*changed.*selection.*clipboard" "Android runtime behavior"
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^BuildId: [0-9]+$" "Android build id"
require_device_marker "docs/native-textinput-evidence/android-runtime.md" "Android device" android
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^Transcript: .+" "Android transcript path"
require_marker_file "docs/native-textinput-evidence/android-runtime.md" "Transcript" "Android transcript file"
require_marker_file_pattern "docs/native-textinput-evidence/android-runtime.md" "Transcript" 'AppStarted|RunViewCreated' "Android transcript content"
require_pattern "docs/native-textinput-evidence/android-runtime.md" "^Command: .*native_textinput_device_runtime_evidence\\.sh" "Android evidence command"
require_not_pattern "docs/native-textinput-evidence/android-runtime.md" "cargo test .*--test ui|makepad_test" "Android evidence must use device helpers"

require_file "docs/native-textinput-evidence/ohos-runtime.md" "OpenHarmony DevEco/device runtime"
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^Status: PASS$" "OpenHarmony status"
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^RunItem: makepad-example-native-text-input-ohos$" "OpenHarmony RunItem"
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^Example: examples/native_text_input$" "OpenHarmony example path"
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^Verified: .*DevEco.*changed.*selection.*clipboard" "OpenHarmony runtime behavior"
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^BuildId: [0-9]+$" "OpenHarmony build id"
require_device_marker "docs/native-textinput-evidence/ohos-runtime.md" "OpenHarmony device" ohos
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^Transcript: .+" "OpenHarmony transcript path"
require_marker_file "docs/native-textinput-evidence/ohos-runtime.md" "Transcript" "OpenHarmony transcript file"
require_marker_file_pattern "docs/native-textinput-evidence/ohos-runtime.md" "Transcript" 'AppStarted|RunViewCreated' "OpenHarmony transcript content"
require_pattern "docs/native-textinput-evidence/ohos-runtime.md" "^Command: .*native_textinput_device_runtime_evidence\\.sh" "OpenHarmony evidence command"
require_not_pattern "docs/native-textinput-evidence/ohos-runtime.md" "cargo test .*--test ui|makepad_test" "OpenHarmony evidence must use DevEco/device helpers"

if [[ "$missing" -ne 0 ]]; then
    cat >&2 <<'EOF'

NativeTextInput completion audit failed.

Use docs/native-textinput-evidence/README.md for the required evidence format.
Do not mark the eight-stage roadmap complete until this script passes in the
environment where Studio, simulators, DevEco, and devices are available.
EOF
    exit 1
fi

echo "NativeTextInput completion audit passed."
