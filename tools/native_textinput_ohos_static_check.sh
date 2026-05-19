#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

index_ets="tools/open_harmony/deveco/entry/src/main/ets/pages/Index.ets"
makepad_ets="tools/open_harmony/deveco/entry/src/main/ets/makepad/makepad.ets"
module_json="tools/open_harmony/deveco/entry/src/main/module.json5"
platform_toml="platform/Cargo.toml"

require_pattern() {
    local file="$1"
    local pattern="$2"
    local description="$3"
    if ! rg -q "$pattern" "$file"; then
        echo "missing ${description}: ${pattern} in ${file}" >&2
        exit 1
    fi
}

require_pattern "$index_ets" "import \\{ pasteboard \\} from '@kit\\.BasicServicesKit'" "pasteboard import"
require_pattern "$index_ets" "controller: new TextInputController\\(\\)" "per-input TextInputController"
require_pattern "$index_ets" "TextInput\\(\\{ text: input\\.text, placeholder: input\\.placeholder, controller: input\\.controller \\}\\)" "TextInput controller binding"
require_pattern "$index_ets" "input\\.controller\\.setTextSelection\\(" "controller selection update"
require_pattern "$index_ets" "pasteboard\\.createData\\(pasteboard\\.MIMETYPE_TEXT_PLAIN, text\\)" "clipboard write"
require_pattern "$index_ets" "pasteboard\\.getSystemPasteboard\\(\\)\\.setDataSync\\(data\\)" "sync pasteboard set"
require_pattern "$index_ets" "systemPasteboard\\.getDataSync\\(\\)\\.getPrimaryText\\(\\)" "sync pasteboard text read"
require_pattern "$index_ets" "handleNativeTextInputChanged\\(input\\.inputId, nextText\\)" "changed callback after cut/paste"
require_pattern "$index_ets" "handleNativeTextInputSelectionChanged\\(input\\.inputId, nextSelection, nextSelection\\)" "selection callback after cut/paste"
require_pattern "$index_ets" "handleNativeTextInputSelectionChanged\\(input\\.inputId, nextStart, nextEnd\\)" "selection callback after select all"
require_pattern "$index_ets" "focusControl\\.requestFocus\\('native_text_input_' \\+ input\\.inputId\\)" "focus before native command"

require_pattern "$makepad_ets" "controller: TextInputController" "NativeTextInputState controller field"
require_pattern "$makepad_ets" "selectionStart: number" "NativeTextInputState selectionStart"
require_pattern "$makepad_ets" "selectionEnd: number" "NativeTextInputState selectionEnd"
require_pattern "$makepad_ets" "handleNativeTextInputSelectionChanged\\(inputId: string, start: number, end: number\\): void" "selection callback interface"

require_pattern "$module_json" "ohos\\.permission\\.READ_PASTEBOARD" "pasteboard read permission"
require_pattern "$platform_toml" "hilog-sys = \"=0\\.1\\.7\"" "pinned hilog-sys version"
require_pattern "$platform_toml" "napi-derive-ohos = \"=0\\.0\\.9\"" "pinned napi derive version"
require_pattern "$platform_toml" "napi-ohos = \"=0\\.1\\.3\"" "pinned napi runtime version"
require_pattern "$platform_toml" "ohos-sys = \\{ version = \"=0\\.2\\.2\"" "pinned ohos-sys version"

echo "OpenHarmony NativeTextInput static contract check passed."
