#!/usr/bin/env bash
set -euo pipefail

# Build, sign, install and launch a Makepad app on a REAL HarmonyOS device.
#
# Differences from tools/ohos_sim_sign_run.sh (the emulator path):
#   1. NO MAKEPAD=ohos_sim — real devices must use the normal shader compile
#      path; the tolerant "park failed shaders" emulator mode would silently
#      mask real shader bugs. This script refuses to run with it set.
#   2. Real signing material — a commercial HarmonyOS device (e.g. 6.1.x)
#      rejects the OpenHarmony SDK community signature at bm install time.
#      You need AGC (AppGallery Connect) debug material, easiest obtained by
#      opening the generated DevEco project once and using
#      File > Project Structure > Signing Configs > Automatically generate
#      signature (this registers the device UDID in AGC), then exporting or
#      pointing this script at the material via env vars:
#        OHOS_SIGN_P12       path to keystore .p12
#        OHOS_SIGN_P12_PWD   keystore password
#        OHOS_SIGN_ALIAS     key alias                (default: debugKey)
#        OHOS_SIGN_KEY_PWD   key password             (default: $OHOS_SIGN_P12_PWD)
#        OHOS_SIGN_CERT      app debug certificate .cer
#        OHOS_SIGN_PROFILE   signed provision profile .p7b
#   3. Device gate — refuses emulator targets (127.0.0.1:*); expects a USB
#      device in `hdc list targets`.

DEVECO_HOME="${DEVECO_HOME:-/Applications/DevEco-Studio.app/Contents}"
CRATE="makepad-example-native-text-input"
SKIP_BUILD=0

usage() {
    cat <<'EOF'
Usage: tools/ohos_device_run.sh [options]

Options:
  -p CRATE      Cargo package name (default makepad-example-native-text-input).
  --skip-build  Skip the cargo-makepad build, only sign/install/launch.
  -h, --help    Show this help.

Signing env vars (see header comment): OHOS_SIGN_P12, OHOS_SIGN_P12_PWD,
OHOS_SIGN_ALIAS, OHOS_SIGN_KEY_PWD, OHOS_SIGN_CERT, OHOS_SIGN_PROFILE.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -p) CRATE="${2:?missing value for -p}"; shift 2 ;;
        --skip-build) SKIP_BUILD=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

cd "$(dirname "$0")/.."

JAVA="$DEVECO_HOME/jbr/Contents/Home/bin/java"
LIB="$DEVECO_HOME/sdk/default/openharmony/toolchains/lib"
HDC="$DEVECO_HOME/sdk/default/openharmony/toolchains/hdc"
SIGN_JAR="$LIB/hap-sign-tool.jar"

for f in "$JAVA" "$HDC" "$SIGN_JAR"; do
    if [[ ! -e "$f" ]]; then
        echo "missing required file: $f" >&2
        exit 1
    fi
done

# --- Gate 1: never build a device HAP in emulator-tolerant shader mode. ---
if [[ "${MAKEPAD:-}" == *"ohos_sim"* ]]; then
    echo "ERROR: MAKEPAD=ohos_sim is set. That is the emulator-only tolerant" >&2
    echo "shader mode and must never run on a real device. Unset it:" >&2
    echo "  unset MAKEPAD" >&2
    exit 1
fi

# --- Gate 2: a real device must be connected (not the emulator loopback). ---
TARGETS="$("$HDC" list targets 2>/dev/null | tr -d '\r' | grep -v '^\[Empty\]' || true)"
if [[ -z "$TARGETS" ]]; then
    echo "ERROR: no device in 'hdc list targets'. Connect the phone via USB," >&2
    echo "enable Developer options + USB debugging, and accept the prompt." >&2
    exit 1
fi
if echo "$TARGETS" | grep -q '^127\.0\.0\.1:'; then
    echo "ERROR: only an emulator target (127.0.0.1:*) is connected." >&2
    echo "This script is for real devices; use tools/ohos_sim_sign_run.sh for the emulator." >&2
    exit 1
fi
echo "device: $TARGETS"

# --- Signing material. ---
OHOS_SIGN_ALIAS="${OHOS_SIGN_ALIAS:-debugKey}"
OHOS_SIGN_KEY_PWD="${OHOS_SIGN_KEY_PWD:-${OHOS_SIGN_P12_PWD:-}}"
MISSING=""
for v in OHOS_SIGN_P12 OHOS_SIGN_P12_PWD OHOS_SIGN_CERT OHOS_SIGN_PROFILE; do
    if [[ -z "${!v:-}" ]]; then MISSING="$MISSING $v"; fi
done
if [[ -n "$MISSING" ]]; then
    echo "ERROR: missing signing env vars:$MISSING" >&2
    echo "" >&2
    echo "Get AGC debug material once via DevEco Studio:" >&2
    echo "  1. open the generated project (target/makepad-open-harmony/<crate>)" >&2
    echo "     or tools/open_harmony/deveco in DevEco Studio" >&2
    echo "  2. sign in with your Huawei developer account" >&2
    echo "  3. File > Project Structure > Signing Configs" >&2
    echo "     > check 'Automatically generate signature'" >&2
    echo "  4. point the env vars at the generated .p12/.cer/.p7b" >&2
    echo "     (shown in the Signing Configs dialog, usually under ~/.ohos/config)" >&2
    exit 1
fi
for f in "$OHOS_SIGN_P12" "$OHOS_SIGN_CERT" "$OHOS_SIGN_PROFILE"; do
    if [[ ! -f "$f" ]]; then
        echo "missing signing file: $f" >&2
        exit 1
    fi
done

UNDERSCORE_CRATE="${CRATE//-/_}"
BUNDLE="dev.makepad.${UNDERSCORE_CRATE}"
PRJ="target/makepad-open-harmony/${UNDERSCORE_CRATE}"
OUT_DIR="$PRJ/entry/build/default/outputs/default"
UNSIGNED_HAP="$OUT_DIR/makepad-default-unsigned.hap"
SIGNED_HAP="$OUT_DIR/makepad-default-device-signed.hap"

# --- Build (normal shader path, release). ---
if [[ "$SKIP_BUILD" -eq 0 ]]; then
    echo "building $CRATE (no ohos_sim)..."
    env -u MAKEPAD cargo run -p cargo-makepad -- ohos \
        --deveco-home="$DEVECO_HOME" build -p "$CRATE" --release
fi
if [[ ! -f "$UNSIGNED_HAP" ]]; then
    echo "missing $UNSIGNED_HAP (build failed or --skip-build without a prior build?)" >&2
    exit 1
fi

# --- Sign with the AGC material. ---
"$JAVA" -jar "$SIGN_JAR" sign-app \
    -keyAlias "$OHOS_SIGN_ALIAS" \
    -signAlg SHA256withECDSA \
    -mode localSign \
    -appCertFile "$OHOS_SIGN_CERT" \
    -profileFile "$OHOS_SIGN_PROFILE" \
    -inFile "$UNSIGNED_HAP" \
    -keystoreFile "$OHOS_SIGN_P12" \
    -outFile "$SIGNED_HAP" \
    -keyPwd "$OHOS_SIGN_KEY_PWD" \
    -keystorePwd "$OHOS_SIGN_P12_PWD"
echo "signed: $SIGNED_HAP"

# --- Install and launch, mirroring cargo-makepad's run() hdc sequence. ---
BUNDLE_DIR="data/local/tmp/${UNDERSCORE_CRATE}"
"$HDC" shell aa force-stop "$BUNDLE" || true
"$HDC" shell rm -rf "$BUNDLE_DIR" || true
"$HDC" shell mkdir "$BUNDLE_DIR"
"$HDC" file send "$SIGNED_HAP" "$BUNDLE_DIR"
"$HDC" shell bm install -p "$BUNDLE_DIR"
"$HDC" shell rm -rf "$BUNDLE_DIR"
"$HDC" shell aa start -a EntryAbility -b "$BUNDLE"
echo "launched: $BUNDLE"
echo "logs: \"$HDC\" shell hilog | grep -E 'Makepad|makepad'"
