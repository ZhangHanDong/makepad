Status: PASS (simulator slice core chain; Makepad self-render deferred, see below)
RunItem: makepad-example-native-text-input-ohos
Example: examples/native_text_input
Verified: install launch module_init xcomponent_surface egl_context create layout attach focus blur changed selection echo_guard
BuildId: local-cli (Studio bridge not used for this bring-up pass)
Device: OpenHarmony local simulator (127.0.0.1:5555, aarch64, API 15, emulator)
Transcript: docs/native-textinput-evidence/ohos-runtime.log
Command: MAKEPAD=ohos_sim cargo run -p cargo-makepad -- ohos --deveco-home="$DEVECO_HOME" build -p makepad-example-native-text-input --release && tools/ohos_sim_sign_run.sh

DevEcoHome: /Applications/DevEco-Studio.app/Contents
JavaHome: /Applications/DevEco-Studio.app/Contents/jbr/Contents/Home
HdcTargets: 127.0.0.1:5555
Hap: target/makepad-open-harmony/makepad_example_native_text_input/entry/build/default/outputs/default/makepad-default-signed.hap 62422599 sha256:3ccab765a1771698...
Screenshot: docs/native-textinput-evidence/ohos-first-run.jpeg (three ArkUI TextInput overlays visible at Makepad-computed positions)

LayerTrace:
  RustOp: PASS (CreateNativeView/UpdateNativeViewLayout emitted from widget tree)
  OhosPlatformHandler: PASS (open_harmony.rs maps ops to ArkGlue NAPI calls)
  ArkTsHostMethod: PASS ([MakepadNTI] create/layout traces, ids 21/23/25/58)
  ArkTsState: PASS (attached flags flip via onAppear, text state updates)
  ArkUiVisual: PASS (screenshot: "Type here" / "Secondary native field" /
    "Scroll/clipped host" rendered at expected positions)
  ArkTsCallback: PASS (onChange/onFocus/onBlur/onTextSelectionChange fire)
  RustAction: PASS (example handler logs: "changed action received: hello
    ohos", "focus action received", "blur action received", "selection 10..10")
  MakepadStatusLabel: DEFERRED (Makepad self-rendered UI is invisible on this
    emulator; see GPU limitation below — actions verified via hilog instead)
LayoutTrace:
  23: makepad_rect=38,162,311.69,36 arkui_global={"x":38,"y":200.77} arkui_size=311.69x36 vp_px=3.25
  25: makepad_rect=38,230,311.69,36 arkui_global={"x":38,"y":268.77} arkui_size=311.69x36 vp_px=3.25
  58: makepad_rect=46,668,295.69,36 arkui_global={"x":46,"y":706.77} arkui_size=295.69x36 vp_px=3.25
  Note: Makepad logical units map 1:1 to ArkUI vp on this device
  (dpi_factor == display density 3.25). ArkUI global y = makepad y + ~38.77vp
  constant offset (XComponent sits below the status bar inside the window).
CommandTrace:
  23: command=changed queued=false programmatic_echo=false len=8/9/10
  (typed "hello ohos" via uitest; per-keystroke onChange flowed through the
  echo guard as user edits and reached the Rust widget action handler)

## Verified Chain (hilog markers in transcript)

```
uitest inputText "hello ohos"
  -> ArkUI TextInput onChange (per keystroke)
  -> [MakepadNTI] changed id=23 programmatic_echo=false len=8/9/10
  -> handleNativeTextInputChanged -> oh_callbacks.rs
  -> NativeTextInputChanged action
  -> example main.rs: "changed action received: hello ohos"
  -> selection sync: "selection 10..10"
```

Also verified: focus action on tap, blur action when the IME first-run dialog
stole focus, selection 0..0 on startup, create->attached(onAppear)->flush
ordering for all three inputs, and the per-input layout trace above.

Simulator quirk: the first run requires accepting the Celia IME (小艺输入法)
privacy dialog and keyboard-layout wizard before typed input reaches the
field; before that, taps focus the field (focus action arrives) but no
changed events fire.

## Emulator GPU Limitation (still true, now tolerated)

The emulator reports `GL_VENDOR=ARM GL_RENDERER=Mali-G77 GL_VERSION=OpenGL ES
3.0 (4.1 Metal - 88.1)`: guest GLES3 shaders are translated to desktop GLSL
450 while the macOS host OpenGL caps at 4.1, so every Makepad shader fails to
compile (errors preserved in the transcript). Previously this panicked the
render thread before any native op was emitted. Under `MAKEPAD=ohos_sim` the
GL backend now parks failed shaders in `GlShaderState::Failed`, logs
`SHADER::*::COMPILATION_FAILED (skipping shader under ohos_sim tolerant
mode)`, skips those draw items, and keeps the main loop alive — so the ArkUI
overlay host is fully exercisable while Makepad's own pixels stay blank.
Non-`ohos_sim` builds (real devices, desktop, Android) keep the original
panic behavior. Makepad self-render on OHOS therefore still requires the real
device gate; the two prior cppcrash records
(`cppcrash-dev.makepad.makepad_example_native_text_input-20020051-20260610214512/21`)
document the pre-tolerance aborts.

## Defects Found and Fixed During Bring-up

1. `Index.ets` XComponent `libraryname: 'entry'` pointed at a nonexistent
   libentry.so, so the XComponent never delivered the surface to Rust (black
   screen, zero logs). Fixed to `'makepad'`.
2. The OHOS entry path never called `Cx::init_log()` (desktop/Android/wasm
   do), so all Rust-side `crate::log!` output was silently dropped. Fixed in
   `app_main.rs`.
3. `bm install` error 9568289: `READ_PASTEBOARD` is a system_basic ACL
   permission and must be listed in the provision profile `acls.allowed-acls`.
   Handled by `tools/ohos_sim_sign_run.sh`.
4. cargo-makepad's `run` requires `makepad-default-signed.hap` but the
   generated project has no signingConfigs. `tools/ohos_sim_sign_run.sh` signs
   with the SDK's OpenHarmony community material (bundle name, simulator UDID,
   fresh validity, the ACL above) and mirrors the install/launch sequence.
5. Emulator GL panic on shader compile, fixed by the `ohos_sim` tolerant mode
   described above.

Operational note: this emulator instance crashed twice during the session
(qemu process exits, `hdc list targets` goes `[Empty]`; reconnect via DevEco
Device Manager restart, then `hdc tconn 127.0.0.1:5555`).

## Remaining for Full P5

- Manual checklist items not yet driven: Set Text / Set Both / Select All /
  Copy / Cut / Paste buttons (Makepad-drawn buttons are invisible on the
  emulator; they are hit-testable blind, or validate on a real device),
  programmatic set_text echo-guard runtime proof, detach/close/relaunch
  lifecycle sweep, callback-order notes for paste/cut.
- DevEco ArkTS compile validation is implicitly covered (hvigor release build
  passes with the new host code).
- Real-device run for Makepad self-render, IME/emoji, clipboard
  permission prompts, and the rest of the device gate.
