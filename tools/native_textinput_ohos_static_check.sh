#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

index_ets="tools/open_harmony/deveco/entry/src/main/ets/pages/Index.ets"
makepad_ets="tools/open_harmony/deveco/entry/src/main/ets/makepad/makepad.ets"
module_json="tools/open_harmony/deveco/entry/src/main/module.json5"
platform_toml="platform/Cargo.toml"
ohos_compile_rs="tools/cargo_makepad/src/open_harmony/compile.rs"
ohos_rs="platform/src/os/linux/open_harmony/open_harmony.rs"
native_input_rs="widgets/src/native_text_input.rs"
native_label_rs="widgets/src/native_label.rs"

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
require_pattern "$index_ets" "existing\\.controller : new TextInputController\\(\\)" "per-input TextInputController with reuse on re-create"
require_pattern "$index_ets" "TextInput\\(\\{ text: this\\.state\\.text, placeholder: this\\.state\\.placeholder, controller: this\\.state\\.controller \\}\\)" "TextInput controller binding"
require_pattern "$index_ets" "@ObjectLink state: NativeTextInputState" "ObjectLink item state (ForEach reuse fix)"
require_pattern "$makepad_ets" "@Observed" "Observed state class"
require_pattern "$makepad_ets" "class NativeLabelState" "NativeLabelState class"
require_pattern "$makepad_ets" "setNativeLabelText\(inputId: string, text: string\): void" "label text host method"
require_pattern "$index_ets" "@ObjectLink state: NativeLabelState" "ObjectLink label state"
require_pattern "$index_ets" "hitTestBehavior\\(HitTestMode\\.None\\)" "native label click-through"
require_pattern "$index_ets" "createNativeLabel" "label create host method"
require_pattern "$index_ets" "input\\.controller\\.setTextSelection\\(" "controller selection update"
require_pattern "$index_ets" "pasteboard\\.createData\\(pasteboard\\.MIMETYPE_TEXT_PLAIN, text\\)" "clipboard write"
require_pattern "$index_ets" "pasteboard\\.getSystemPasteboard\\(\\)\\.setDataSync\\(data\\)" "sync pasteboard set"
require_pattern "$index_ets" "systemPasteboard\\.getDataSync\\(\\)\\.getPrimaryText\\(\\)" "sync pasteboard text read"
require_pattern "$index_ets" "handleNativeTextInputChanged\\(input\\.inputId, nextText\\)" "changed callback after cut/paste"
require_pattern "$index_ets" "handleNativeTextInputSelectionChanged\\(input\\.inputId, nextSelection, nextSelection\\)" "selection callback after cut/paste"
require_pattern "$index_ets" "handleNativeTextInputSelectionChanged\\(input\\.inputId, nextStart, nextEnd\\)" "selection callback after select all"
require_pattern "$index_ets" "private requestNativeFocus\\(focusId: string\\): void" "native focus request hook"
require_pattern "$index_ets" "focusControl\\.requestFocus\\(focusId\\)" "native focusControl request"
require_pattern "$index_ets" "this\\.requestNativeFocus\\('native_text_input_' \\+ input\\.inputId\\)" "focus before native command"
require_pattern "$index_ets" "\\[MakepadNTI\\]" "layout/command trace prefix"
require_pattern "$index_ets" "pendingProgrammaticEchoes" "programmatic echo guard"
require_pattern "$index_ets" "lastProgrammaticText" "programmatic echo text compare"
require_pattern "$index_ets" "pendingCommands" "command queue before attach"
require_pattern "$index_ets" "input\\.controller\\.stopEditing\\(\\)" "controller blur path"
require_pattern "$index_ets" "\\.onAppear\\(" "attach flush hook"
require_pattern "$index_ets" "\\.onAreaChange\\(" "layout trace area hook"
require_pattern "$index_ets" "vp2px\\(1\\)" "vp/px ratio trace"

require_pattern "$makepad_ets" "controller: TextInputController" "NativeTextInputState controller field"
require_pattern "$makepad_ets" "pendingProgrammaticEchoes: number" "NativeTextInputState echo count"
require_pattern "$makepad_ets" "attached: boolean" "NativeTextInputState attach flag"
require_pattern "$makepad_ets" "selectionStart: number" "NativeTextInputState selectionStart"
require_pattern "$makepad_ets" "selectionEnd: number" "NativeTextInputState selectionEnd"
require_pattern "$makepad_ets" "handleNativeTextInputSelectionChanged\\(inputId: string, start: number, end: number\\): void" "selection callback interface"

require_pattern "$module_json" "ohos\\.permission\\.READ_PASTEBOARD" "pasteboard read permission"
require_pattern "$platform_toml" "hilog-sys = \"=0\\.1\\.7\"" "pinned hilog-sys version"
require_pattern "$platform_toml" "napi-derive-ohos = \"=0\\.0\\.9\"" "pinned napi derive version"
require_pattern "$platform_toml" "napi-ohos = \"=0\\.1\\.3\"" "pinned napi runtime version"
require_pattern "$platform_toml" "ohos-sys = \\{ version = \"=0\\.2\\.2\"" "pinned ohos-sys version"
require_pattern "$ohos_compile_rs" "sync_deveco_runtime_template\\(&prj_path\\)" "generated DevEco runtime template refresh"
require_pattern "$ohos_rs" "native_host_layouts" "resolved native host layout cache"
require_pattern "$ohos_rs" "native_host_layouts\\.get\\(&id\\) == Some\\(&\\(rect, shown\\)\\)" "post-draw native layout dedup"
require_pattern "$native_input_rs" '#\[cfg\(target_env = "ohos"\)\]' "OHOS text input Area queue"
require_pattern "$native_label_rs" '#\[cfg\(target_env = "ohos"\)\]' "OHOS label Area queue"

echo "OpenHarmony NativeTextInput static contract check passed."
