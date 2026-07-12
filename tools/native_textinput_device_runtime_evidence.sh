#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

studio_addr="127.0.0.1:8001"
platform=""
device=""
verified=""
timeout=180
allow_no_clear=0
clear_ids=()
command_line="$0 $*"
self_test=0

usage() {
    cat <<'EOF'
Usage: tools/native_textinput_device_runtime_evidence.sh --platform ios|android|ohos [options]

Launches the checked-in NativeTextInput example through a Studio RunItem and
writes the matching runtime evidence file only after the operator explicitly
records the verified behavior. This script does not automate IME, soft
keyboard, or pasteboard behavior; it makes the required Studio launch path and
evidence marker reproducible.

Options:
  --platform NAME        Required. One of ios, android, ohos.
  --studio ADDR         Studio bridge target, default 127.0.0.1:8001.
  --clear-build-id ID   Send ClearBuild for a previous build id before RunItem.
                        Repeat to clear multiple build tabs.
  --allow-no-clear      Allow RunItem without ClearBuild. Use only when
                        ListBuilds shows no old build for this target.
  --device TEXT         Required. Device/simulator/emulator identifier.
  --verified TEXT       Required. Must contain the platform-specific checked
                        words listed below.
  --timeout SECONDS     Startup timeout, default 180.
  --self-test           Test argument guards without connecting to Studio.
  -h, --help            Show this help.

Required --verified words:
  ios:     keyboard changed selection clipboard
  android: IME changed selection clipboard
  ohos:    DevEco changed selection clipboard

Examples:
  tools/native_textinput_device_runtime_evidence.sh \
    --platform ios --clear-build-id 12 \
    --device "iPhone 16 Simulator (UDID 00008110-001234D20E91801E)" \
    --verified "keyboard changed selection clipboard"

  tools/native_textinput_device_runtime_evidence.sh \
    --platform android --clear-build-id 13 \
    --device "Pixel 8 API 35 emulator (serial emulator-5554)" \
    --verified "IME changed selection clipboard"
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --platform)
            platform="${2:?missing value for --platform}"
            shift 2
            ;;
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
        --device)
            device="${2:?missing value for --device}"
            shift 2
            ;;
        --verified)
            verified="${2:?missing value for --verified}"
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

run_self_test() {
    local build_line='{"BuildStarted":{"build_id":[77],"mount":"makepad","package":"makepad-example-native-text-input-ios-sim"}}'
    local app_line='{"AppStarted":{"build_id":[77]}}'
    local run_view_line='{"RunViewCreated":{"build_id":[77],"kind_id":0}}'

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
    if [[ "$(extract_build_id "$build_line")" != "77" ]]; then
        echo "self-test failed: build id parser returned the wrong id" >&2
        exit 1
    fi
    if ! is_app_connection_line "$app_line" 77; then
        echo "self-test failed: AppStarted line was not accepted" >&2
        exit 1
    fi
    if ! is_app_connection_line "$run_view_line" 77; then
        echo "self-test failed: RunViewCreated line was not accepted" >&2
        exit 1
    fi
    if is_app_connection_line "$app_line" 76; then
        echo "self-test failed: AppStarted line matched the wrong build id" >&2
        exit 1
    fi

    echo "NativeTextInput device evidence self-test passed."
}

if [[ "$self_test" -eq 1 ]]; then
    run_self_test
    exit 0
fi

case "$platform" in
    ios)
        target_name="makepad-example-native-text-input-ios-sim"
        output="docs/native-textinput-evidence/ios-runtime.md"
        required_words=(keyboard changed selection clipboard)
        ;;
    android)
        target_name="makepad-example-native-text-input-android"
        output="docs/native-textinput-evidence/android-runtime.md"
        required_words=(IME changed selection clipboard)
        ;;
    ohos)
        target_name="makepad-example-native-text-input-ohos"
        output="docs/native-textinput-evidence/ohos-runtime.md"
        required_words=(DevEco changed selection clipboard)
        ;;
    "")
        echo "missing required --platform" >&2
        usage >&2
        exit 2
        ;;
    *)
        echo "unknown platform: $platform" >&2
        usage >&2
        exit 2
        ;;
esac

if [[ "${#clear_ids[@]}" -eq 0 && "$allow_no_clear" -eq 0 ]]; then
    echo "refusing to RunItem without --clear-build-id or --allow-no-clear" >&2
    echo "run tools/native_textinput_studio_gate.sh --list-builds first" >&2
    exit 2
fi

if [[ -z "$verified" ]]; then
    echo "missing required --verified evidence words" >&2
    usage >&2
    exit 2
fi

if [[ -z "$device" ]]; then
    echo "missing required --device" >&2
    usage >&2
    exit 2
fi

if is_placeholder_device "$platform" "$device"; then
    echo "invalid --device placeholder for $platform: $device" >&2
    echo "pass the actual simulator/device name plus UDID, serial, or DevEco target identifier" >&2
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

send_json '{"ListBuilds":[]}'
wait_for_line '"Builds"' "ListBuilds response" >/dev/null

for id in "${clear_ids[@]}"; do
    send_json "{\"ClearBuild\":{\"build_id\":[$id]}}"
done

send_json "{\"RunItem\":{\"mount\":\"makepad\",\"name\":\"$target_name\"}}"
started_line="$(wait_for_line '"BuildStarted"|"AppStarted"|"RunViewCreated"' "$target_name startup")"
build_id="$(extract_build_id "$started_line")"
if ! is_app_connection_line "$started_line" "$build_id"; then
    wait_for_line "\"AppStarted\".*\"build_id\":\\[$build_id\\]|\"RunViewCreated\".*\"build_id\":\\[$build_id\\]" "$target_name app connection" >/dev/null
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
Device: $device
Transcript: $transcript
Command: $command_line

Operator evidence:
- Studio RunItem reached AppStarted or RunViewCreated for this build.
- Operator verified the platform behavior named in the Verified marker.
EOF

echo "wrote $output"
