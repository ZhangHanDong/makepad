#!/usr/bin/env bash
set -euo pipefail

# Sign the cargo-makepad OHOS unsigned HAP with the OpenHarmony SDK default
# signing material, then install and launch it on the connected simulator.
#
# Why this exists: `cargo makepad ohos run` builds
# entry/build/default/outputs/default/makepad-default-unsigned.hap and then
# requires makepad-default-signed.hap, but the generated DevEco project has no
# signingConfigs. This script fills that gap with the SDK's community signing
# material (OpenHarmony.p12 / OpenHarmonyProfileDebug.pem, password 123456):
#   1. build the app certificate chain from the cert embedded in the debug
#      profile template plus the CA certs exported from the keystore;
#   2. rewrite the debug profile template with the real bundle name, the
#      target device UDID, and a fresh validity window, then sign it;
#   3. sign the unsigned HAP;
#   4. mirror cargo-makepad's install/launch hdc sequence.
#
# Note: a HarmonyOS-image emulator may reject OpenHarmony community
# signatures at `bm install` time. If that happens, record the exact bm error
# as evidence and fall back to DevEco auto-signing of the generated project.

DEVECO_HOME="${DEVECO_HOME:-/Applications/DevEco-Studio.app/Contents}"
CRATE="makepad-example-native-text-input"
UDID=""
SIGN_ONLY=0

usage() {
    cat <<'EOF'
Usage: tools/ohos_sim_sign_run.sh [options]

Options:
  -p CRATE      Cargo package name (default makepad-example-native-text-input).
  --udid UDID   Device UDID for the debug profile. Default: read from
                `hdc shell bm get --udid`.
  --sign-only   Sign the HAP but skip install/launch.
  -h, --help    Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -p) CRATE="${2:?missing value for -p}"; shift 2 ;;
        --udid) UDID="${2:?missing value for --udid}"; shift 2 ;;
        --sign-only) SIGN_ONLY=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

cd "$(dirname "$0")/.."

JAVA="$DEVECO_HOME/jbr/Contents/Home/bin/java"
KEYTOOL="$DEVECO_HOME/jbr/Contents/Home/bin/keytool"
LIB="$DEVECO_HOME/sdk/default/openharmony/toolchains/lib"
HDC="$DEVECO_HOME/sdk/default/openharmony/toolchains/hdc"
SIGN_JAR="$LIB/hap-sign-tool.jar"
KEYSTORE="$LIB/OpenHarmony.p12"
PROFILE_CERT="$LIB/OpenHarmonyProfileDebug.pem"
TEMPLATE="$LIB/UnsgnedDebugProfileTemplate.json"
PWD_MATERIAL="123456"

for f in "$JAVA" "$KEYTOOL" "$HDC" "$SIGN_JAR" "$KEYSTORE" "$PROFILE_CERT" "$TEMPLATE"; do
    if [[ ! -e "$f" ]]; then
        echo "missing required file: $f" >&2
        exit 1
    fi
done

UNDERSCORE_CRATE="${CRATE//-/_}"
BUNDLE="dev.makepad.${UNDERSCORE_CRATE}"
PRJ="target/makepad-open-harmony/${UNDERSCORE_CRATE}"
OUT_DIR="$PRJ/entry/build/default/outputs/default"
UNSIGNED_HAP="$OUT_DIR/makepad-default-unsigned.hap"
SIGNED_HAP="$OUT_DIR/makepad-default-signed.hap"
WORK="$PRJ/signing"

if [[ ! -f "$UNSIGNED_HAP" ]]; then
    echo "missing $UNSIGNED_HAP" >&2
    echo "build it first: MAKEPAD=ohos_sim cargo run -p cargo-makepad -- ohos --deveco-home=\"$DEVECO_HOME\" build -p $CRATE --release" >&2
    exit 1
fi

if [[ -z "$UDID" && "$SIGN_ONLY" -eq 0 ]] || [[ -z "$UDID" ]]; then
    UDID="$("$HDC" shell bm get --udid 2>/dev/null | tr -d '\r' | grep -E '^[0-9A-Fa-f]{40,}$' | head -1 || true)"
    if [[ -z "$UDID" ]]; then
        echo "failed to read device UDID from hdc; is the simulator connected?" >&2
        echo "check: $HDC list targets" >&2
        exit 1
    fi
fi
echo "udid: $UDID"

mkdir -p "$WORK"

# 1. App certificate chain: leaf cert embedded in the profile template,
#    sub/root CA certs exported from the default keystore.
python3 - "$TEMPLATE" "$WORK/app_leaf.pem" <<'PY'
import json, sys
template = json.load(open(sys.argv[1]))
open(sys.argv[2], 'w').write(template['bundle-info']['development-certificate'])
PY
"$KEYTOOL" -exportcert -rfc -keystore "$KEYSTORE" -storepass "$PWD_MATERIAL" -storetype pkcs12 \
    -alias "openharmony application ca" -file "$WORK/app_subca.pem" >/dev/null
"$KEYTOOL" -exportcert -rfc -keystore "$KEYSTORE" -storepass "$PWD_MATERIAL" -storetype pkcs12 \
    -alias "openharmony application root ca" -file "$WORK/app_rootca.pem" >/dev/null
cat "$WORK/app_leaf.pem" "$WORK/app_subca.pem" "$WORK/app_rootca.pem" > "$WORK/app_release_chain.pem"

# 2. Debug provision profile: real bundle name, this device's UDID, fresh
#    validity (the SDK template expired in 2024), and the restricted
#    permissions the app requests in module.json5. READ_PASTEBOARD is a
#    system_basic ACL permission; a normal-APL app must list it in
#    acls.allowed-acls or bm install fails with code 9568289
#    (grant request permissions failed).
python3 - "$TEMPLATE" "$WORK/profile.json" "$BUNDLE" "$UDID" <<'PY'
import json, sys, time
template = json.load(open(sys.argv[1]))
template['bundle-info']['bundle-name'] = sys.argv[3]
template['debug-info']['device-ids'] = [sys.argv[4]]
template['acls']['allowed-acls'] = ['ohos.permission.READ_PASTEBOARD']
now = int(time.time())
template['validity']['not-before'] = now - 86400
template['validity']['not-after'] = now + 86400 * 3650
json.dump(template, open(sys.argv[2], 'w'), indent=4)
PY

"$JAVA" -jar "$SIGN_JAR" sign-profile \
    -keyAlias "openharmony application profile debug" \
    -signAlg SHA256withECDSA \
    -mode localSign \
    -profileCertFile "$PROFILE_CERT" \
    -inFile "$WORK/profile.json" \
    -keystoreFile "$KEYSTORE" \
    -outFile "$WORK/profile.p7b" \
    -keyPwd "$PWD_MATERIAL" \
    -keystorePwd "$PWD_MATERIAL"

# 3. Sign the HAP.
"$JAVA" -jar "$SIGN_JAR" sign-app \
    -keyAlias "openharmony application release" \
    -signAlg SHA256withECDSA \
    -mode localSign \
    -appCertFile "$WORK/app_release_chain.pem" \
    -profileFile "$WORK/profile.p7b" \
    -inFile "$UNSIGNED_HAP" \
    -keystoreFile "$KEYSTORE" \
    -outFile "$SIGNED_HAP" \
    -keyPwd "$PWD_MATERIAL" \
    -keystorePwd "$PWD_MATERIAL"

echo "signed: $SIGNED_HAP"
if [[ "$SIGN_ONLY" -eq 1 ]]; then
    exit 0
fi

# 4. Install and launch, mirroring cargo-makepad's run() hdc sequence.
BUNDLE_DIR="data/local/tmp/${UNDERSCORE_CRATE}"
"$HDC" shell aa force-stop "$BUNDLE" || true
"$HDC" shell rm -rf "$BUNDLE_DIR" || true
"$HDC" shell mkdir "$BUNDLE_DIR"
"$HDC" file send "$SIGNED_HAP" "$BUNDLE_DIR"
"$HDC" shell bm install -p "$BUNDLE_DIR"
"$HDC" shell rm -rf "$BUNDLE_DIR"
"$HDC" shell aa start -a EntryAbility -b "$BUNDLE"
echo "launched: $BUNDLE"
