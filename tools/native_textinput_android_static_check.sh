#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

activity_java="tools/cargo_makepad/src/android/java/dev/makepad/android/MakepadActivity.java"
native_java="tools/cargo_makepad/src/android/java/dev/makepad/android/MakepadNative.java"
android_jni_rs="platform/src/os/linux/android/android_jni.rs"
android_rs="platform/src/os/linux/android/android.rs"
cargo_toml="Cargo.toml"

require_pattern() {
    local file="$1"
    local pattern="$2"
    local description="$3"
    if ! rg -q -- "$pattern" "$file"; then
        echo "missing ${description}: ${pattern} in ${file}" >&2
        exit 1
    fi
}

require_pattern "$cargo_toml" "makepad-jni-sys = \\{ path = \"libs/jni-sys\" \\}" "local makepad-jni-sys patch"
require_pattern "$cargo_toml" "makepad-android-state = \\{ path = \"libs/android_state\" \\}" "local makepad-android-state patch"

require_pattern "$native_java" "onNativeTextInputChanged\\(long inputId, String text\\)" "changed native declaration"
require_pattern "$native_java" "onNativeTextInputFocusChanged\\(long inputId, boolean hasFocus\\)" "focus native declaration"
require_pattern "$native_java" "onNativeTextInputSelectionChanged\\(long inputId, int start, int end\\)" "selection native declaration"

require_pattern "$activity_java" "class NativeTextInputView extends EditText" "NativeTextInput EditText subclass"
require_pattern "$activity_java" "private boolean programmatic" "programmatic guard field"
require_pattern "$activity_java" "if \\(!programmatic\\)" "selection programmatic guard"
require_pattern "$activity_java" "MakepadNative\\.onNativeTextInputSelectionChanged\\(inputId, selStart, selEnd\\)" "selection callback"
require_pattern "$activity_java" "private FrameLayout mNativeTextInputOverlay" "native text input overlay"
require_pattern "$activity_java" "private HashMap<Long, NativeTextInputRecord> mNativeTextInputs" "native text input registry"
require_pattern "$activity_java" "mRootLayout\\.addView\\(mNativeTextInputOverlay\\)" "overlay attachment"
require_pattern "$activity_java" "public void createNativeTextInput\\(final long inputId, final String text, final String placeholder, final boolean editable\\)" "create method"
require_pattern "$activity_java" "input\\.addTextChangedListener\\(new TextWatcher\\(\\)" "TextWatcher wiring"
require_pattern "$activity_java" "if \\(!record\\.programmatic\\)" "changed programmatic guard"
require_pattern "$activity_java" "MakepadNative\\.onNativeTextInputChanged\\(inputId, s\\.toString\\(\\)\\)" "changed callback"
require_pattern "$activity_java" "MakepadNative\\.onNativeTextInputFocusChanged\\(inputId, hasFocus\\)" "focus callback"
require_pattern "$activity_java" "public void updateNativeTextInput\\(final long inputId, final int left, final int top, final int right, final int bottom, final boolean visible\\)" "layout update method"
require_pattern "$activity_java" "public void detachNativeTextInput\\(final long inputId\\)" "detach method"
require_pattern "$activity_java" "public void setNativeTextInputText\\(final long inputId, final String text, final boolean programmatic\\)" "set text method"
require_pattern "$activity_java" "record\\.view\\.setProgrammatic\\(programmatic\\)" "view programmatic guard propagation"
require_pattern "$activity_java" "public void setNativeTextInputPlaceholder\\(final long inputId, final String placeholder\\)" "placeholder method"
require_pattern "$activity_java" "public void setNativeTextInputEditable\\(final long inputId, final boolean editable\\)" "editable method"
require_pattern "$activity_java" "public void focusNativeTextInput\\(final long inputId\\)" "focus method"
require_pattern "$activity_java" "public void blurNativeTextInput\\(final long inputId\\)" "blur method"
require_pattern "$activity_java" "public void selectAllNativeTextInput\\(final long inputId\\)" "select all method"
require_pattern "$activity_java" "record\\.view\\.selectAll\\(\\)" "select all command"
require_pattern "$activity_java" "public void copyNativeTextInput\\(final long inputId\\)" "copy method"
require_pattern "$activity_java" "public void cutNativeTextInput\\(final long inputId\\)" "cut method"
require_pattern "$activity_java" "public void pasteNativeTextInput\\(final long inputId\\)" "paste method"
require_pattern "$activity_java" "record\\.view\\.onTextContextMenuItem\\(actionId\\)" "clipboard menu command"
require_pattern "$activity_java" "public void closeNativeTextInput\\(final long inputId\\)" "close method"
require_pattern "$activity_java" "mNativeTextInputs\\.remove\\(inputId\\)" "registry removal"
require_pattern "$activity_java" "mNativeTextInputOverlay\\.removeView\\(record\\.view\\)" "overlay removal"

require_pattern "$android_jni_rs" "Java_dev_makepad_android_MakepadNative_onNativeTextInputChanged" "changed JNI callback"
require_pattern "$android_jni_rs" "NativeTextInputChanged::new\\(input_id, text\\)" "changed action post"
require_pattern "$android_jni_rs" "Java_dev_makepad_android_MakepadNative_onNativeTextInputFocusChanged" "focus JNI callback"
require_pattern "$android_jni_rs" "NativeTextInputFocusChanged::new\\(input_id, has_focus != 0\\)" "focus action post"
require_pattern "$android_jni_rs" "Java_dev_makepad_android_MakepadNative_onNativeTextInputSelectionChanged" "selection JNI callback"
require_pattern "$android_jni_rs" "NativeTextInputSelectionChanged::new\\(input_id, start, end\\)" "selection action post"
require_pattern "$android_jni_rs" "NativeTextInputId::try_from\\(input_id\\)" "typed id conversion"
require_pattern "$android_jni_rs" "\"createNativeTextInput\"" "create Java call"
require_pattern "$android_jni_rs" "\"updateNativeTextInput\"" "update Java call"
require_pattern "$android_jni_rs" "\"detachNativeTextInput\"" "detach Java call"
require_pattern "$android_jni_rs" "\"setNativeTextInputText\"" "set text Java call"
require_pattern "$android_jni_rs" "\"setNativeTextInputPlaceholder\"" "placeholder Java call"
require_pattern "$android_jni_rs" "\"setNativeTextInputEditable\"" "editable Java call"
require_pattern "$android_jni_rs" "\"focusNativeTextInput\"" "focus Java call"
require_pattern "$android_jni_rs" "\"blurNativeTextInput\"" "blur Java call"
require_pattern "$android_jni_rs" "\"selectAllNativeTextInput\"" "select all Java call"
require_pattern "$android_jni_rs" "\"copyNativeTextInput\"" "copy Java call"
require_pattern "$android_jni_rs" "\"cutNativeTextInput\"" "cut Java call"
require_pattern "$android_jni_rs" "\"pasteNativeTextInput\"" "paste Java call"
require_pattern "$android_jni_rs" "\"closeNativeTextInput\"" "close Java call"

require_pattern "$android_rs" "NativeHostProps::TextInput" "Android TextInput create op"
require_pattern "$android_rs" "NativeHostPropUpdate::TextInputText" "Android text prop update"
require_pattern "$android_rs" "NativeHostPropUpdate::TextInputPlaceholder" "Android placeholder prop update"
require_pattern "$android_rs" "NativeHostPropUpdate::TextInputEditable" "Android editable prop update"
require_pattern "$android_rs" "NativeTextInputCommand::SelectAll" "Android select all command dispatch"
require_pattern "$android_rs" "NativeTextInputCommand::Copy" "Android copy command dispatch"
require_pattern "$android_rs" "NativeTextInputCommand::Cut" "Android cut command dispatch"
require_pattern "$android_rs" "NativeTextInputCommand::Paste" "Android paste command dispatch"

echo "Android NativeTextInput static contract check passed."
