spec: task
name: "OHOS NativeTextInput ArkUI Simulator Wrap"
tags: [makepad, openharmony, ohos, arkui, arkts, native-host-widget, text-input, simulator]
---

## Intent

Validate the first OpenHarmony implementation of Makepad `NativeTextInput`
using an ArkUI `TextInput` overlay hosted from ArkTS above the Makepad
`XComponent`.

The implementation should reuse the existing typed native host contract rather
than introducing OHOS-specific widget APIs. The simulator run is an early
runtime gate for P5; it is not a full replacement for later real-device
validation.

## Architecture

The target architecture is:

```text
Makepad NativeTextInput
  -> NativeMountQueue
  -> CxOsOp::CreateNativeView / UpdateNativeViewLayout /
     UpdateNativeViewProps / CommandNativeView / DetachNativeView /
     CloseNativeView
  -> Cx::handle_platform_ops on OHOS
  -> NAPI call into ArkGlue
  -> Index.ets nativeTextInputs state
  -> ArkUI TextInput overlay in Stack above XComponent
  -> ArkTS onChange/onFocus/onBlur/onTextSelectionChange
  -> NAPI callbacks in oh_callbacks.rs
  -> NativeTextInputChanged / FocusChanged / SelectionChanged
  -> NativeTextInput widget actions and Splash on_change
```

The ArkTS overlay route is the required first path. A pure native ArkUI Node
implementation can be investigated later, but it is out of scope for this
simulator slice.

## Validation Model

Runtime validation must be layered. A final screen effect is not enough by
itself, and a log marker is not enough by itself. The accepted observation chain
is:

```text
Rust NativeTextInput API / widget interaction
  -> CxOsOp native view op
  -> OHOS platform handler
  -> NAPI call into ArkGlue
  -> Index.ets host method
  -> nativeTextInputs state update
  -> ArkUI TextInput visual/focus/text state
  -> ArkTS callback
  -> oh_callbacks.rs
  -> Rust NativeTextInput action
  -> Makepad example status label
```

If a behavior fails, the evidence must identify the first layer that did not
advance. Do not patch lower-level OHOS infrastructure before proving that the
ArkUI `TextInput` overlay itself is visible, focusable, and receiving the
expected command.

The Westlake/noice OHOS adapter project is treated as a validation-methodology
reference only. Its adapter-specific mechanisms are explicitly out of scope for
this implementation: smali patches, boot image regeneration, LD_PRELOAD shims,
AOSP InputChannel rewriting, BPF grants, and TLS/network runtime workarounds.

The older `rnoh` project is treated as an ArkUI wrapper reference only. Its
React Native, Fabric, JS bundle, TurboModule, and descriptor registry layers are
not part of this design. The relevant lessons are:

- layout evidence must distinguish Makepad coordinates, ArkUI global position,
  and vp/px conversion;
- focus and blur are ArkUI controller concerns before they are Makepad widget
  API concerns;
- text, selection, paste, and cut callback ordering can be non-obvious and must
  be verified with evidence;
- commands may arrive before the target ArkUI component is attached and must be
  queued or rejected with a visible diagnostic;
- programmatic text updates need a revision or event-count guard so stale native
  commands do not overwrite newer user input;
- keyboard layout work should use OHOS window avoid-area or keyboard-height
  signals when it enters scope.

## User-Facing Behavior

The checked-in example must be used:

```text
examples/native_text_input
```

RunItem:

```text
makepad-example-native-text-input-ohos
```

Expected simulator behavior:

- native ArkUI `TextInput` fields appear where Makepad lays out
  `NativeTextInput` widgets;
- manual text input updates the ArkUI field;
- changed events update Makepad widget state and visible status labels;
- focus and blur commands target the right field;
- programmatic `set_text` updates the field without recursively emitting a
  changed event;
- selection and clipboard commands work for plain text where the simulator
  supports them;
- detach/close removes overlay fields without stale visual remnants.

## Props

The OHOS implementation must support the existing `NativeHostProps::TextInput`
contract:

- `text: String`
- `placeholder: String`
- `editable: bool`

Layout is delivered through:

- `CxOsOp::UpdateNativeViewLayout { id, area, visible }`

The ArkTS side must clamp width/height to valid positive values before applying
them to `TextInput`.

Layout evidence must include, per native input id:

- Makepad rect received by the platform handler;
- ArkUI applied width and height;
- ArkUI global position from the host component when available;
- vp/px ratio used for conversion.

## Commands

The OHOS implementation must map existing commands:

- `focus`
- `blur`
- `select_all`
- `copy`
- `cut`
- `paste`

Command behavior must be scoped by native input id. A command for the primary
field must not mutate the secondary field unless the example explicitly calls a
multi-field action such as `Set Both`.

Commands that reach ArkTS before the target ArkUI input has appeared must not be
silently lost. The host must either queue the command by native input id and
apply it after attachment, or reject it with an evidence log that includes the
native input id, command name, and reason.

Programmatic text commands should carry a revision or event count through the
ArkTS host path. The callback path must use that value, or an equivalent guard,
to avoid treating a local set-text operation as a user edit and to avoid stale
commands overwriting newer manual input.

## Events

The OHOS implementation must post:

- `NativeTextInputChanged { id, text }`
- `NativeTextInputFocusChanged { id, has_focus }`
- `NativeTextInputSelectionChanged { id, start, end }`

Events must enter Rust through best-effort NAPI callbacks and then use
`Cx::try_post_action(...)` so they share the same widget action semantics as
macOS, iOS, and Android.

Event acceptance requires both sides of the bridge:

- ArkTS logs or state must show the callback source event.
- Rust logs or visible Makepad status labels must show that the matching native
  text input id received the action.

A callback observed only in ArkTS is not sufficient. A Rust action observed
without a corresponding ArkUI state change is also not sufficient.

Selection, paste, cut, and text-change ordering must be recorded when those
behaviors are tested. If ArkUI fires selection after text change, or paste/cut
through a separate callback, the evidence must describe the observed order and
the guard used to prevent duplicate or missing `NativeTextInputChanged` events.

## Threading and Ownership Rules

- Rust widget code must not directly call ArkUI.
- Rust platform code may call ArkTS through the retained ArkTS object reference.
- ArkTS owns ArkUI `TextInput` component state.
- Rust owns the typed native host ids and lifecycle intent.
- `detach` hides an ArkUI input but can keep state.
- `close` removes the ArkUI input from `nativeTextInputs`.
- Programmatic text updates must not create a changed-event loop.
- ArkTS owns per-input `TextInputController` or equivalent controller state.
- Blur should first be implemented through the ArkUI controller path, such as
  `stopEditing()`, before changing the shared Rust widget API.
- A hidden focus sink is allowed as an OHOS host workaround if it is documented
  and covered by runtime evidence.

## Implementation Targets

Existing files expected to participate:

- `platform/src/os/linux/open_harmony/open_harmony.rs`
- `platform/src/os/linux/open_harmony/oh_callbacks.rs`
- `tools/open_harmony/deveco/entry/src/main/ets/pages/Index.ets`
- `tools/open_harmony/deveco/entry/src/main/ets/makepad/makepad.ets`
- `tools/open_harmony/deveco/entry/src/main/module.json5`
- `tools/native_textinput_ohos_static_check.sh`
- `makepad.splash`
- `docs/native-textinput-evidence/README.md`

Do not add OHOS-specific `NativeTextInput` widget methods unless the shared
typed native host contract is deliberately changed.

## Environment Requirements

The initial landing environment is a local OpenHarmony simulator:

- DevEco Studio or DevEco command-line tools installed.
- `DEVECO_HOME` set.
- `hdc` available in `PATH`.
- simulator visible in `hdc list targets`.
- `MAKEPAD=ohos_sim` set when building for the simulator.
- Makepad OHOS toolchain installed with `cargo makepad ohos install-toolchain`.

## Build and Run Requirements

Static guard:

```bash
tools/native_textinput_ohos_static_check.sh
```

Target check where dependencies are available:

```bash
cargo check -p makepad-platform --target aarch64-unknown-linux-ohos
```

For x86_64 simulator:

```bash
cargo check -p makepad-platform --target x86_64-unknown-linux-ohos
```

DevEco project generation:

```bash
MAKEPAD=ohos_sim cargo makepad ohos \
  --deveco-home="$DEVECO_HOME" \
  deveco -p makepad-example-native-text-input --release
```

Simulator run:

```bash
MAKEPAD=ohos_sim cargo makepad ohos \
  --deveco-home="$DEVECO_HOME" \
  run -p makepad-example-native-text-input --release
```

Formal runtime validation should use Studio RunItem
`makepad-example-native-text-input-ohos` when Studio is available.

Runtime diagnostics must be captured before local filtering. Avoid using inline
`hdc shell "hilog | grep ..."` as the source of truth because command echo can
contaminate the searched stream on some OHOS setups. Capture raw logs first:

```bash
hdc shell "hilog -x > /data/local/tmp/makepad_native_textinput.log"
hdc file recv /data/local/tmp/makepad_native_textinput.log \
  docs/native-textinput-evidence/ohos-runtime.log
```

Local grep or `rg` over the saved file is acceptable.

## Acceptance Criteria

Simulator slice acceptance:

- `tools/native_textinput_ohos_static_check.sh` passes.
- The DevEco project for `makepad-example-native-text-input` is generated.
- The example installs and launches on the local simulator.
- At least one ArkUI `TextInput` is visible above the Makepad `XComponent`.
- Manual ASCII input changes the native field and posts a changed event.
- Focus/blur can be requested from Makepad controls.
- Programmatic set text updates the native field and does not loop through
  changed events.
- Selection callback reaches Rust for select-all or manual selection.
- Plain-text copy/cut/paste is either verified or documented as a simulator
  limitation with logs.
- Detach/close/relaunch leaves no stale overlay input.
- `docs/native-textinput-evidence/ohos-runtime.md` records the run or records
  the first concrete blocker with command output.
- The evidence identifies the deepest completed layer for any failed item.
- The run includes a screenshot or equivalent Studio visual capture proving the
  actual UI state.

Full P5 remains open until a runtime-capable environment produces completion
audit evidence for the required OHOS device/runtime markers.

## Evidence Format

Use:

```text
docs/native-textinput-evidence/ohos-runtime.md
docs/native-textinput-evidence/ohos-runtime.log
```

Minimum evidence marker:

```text
Status: PASS
RunItem: makepad-example-native-text-input-ohos
Example: examples/native_text_input
Verified: DevEco changed selection clipboard
BuildId: <studio-build-id>
Device: OpenHarmony local simulator (<hdc target>)
Transcript: docs/native-textinput-evidence/ohos-runtime.log
Command: tools/native_textinput_device_runtime_evidence.sh --platform ohos ...
```

Required evidence fields:

```text
DevEcoHome: <path>
JavaHome: <path>
HdcTargets: <raw hdc list targets output>
Hap: <path> <bytes> <checksum>
LibMakepad: <path> <bytes> <checksum>
Screenshot: <path>
LayerTrace:
  RustOp: PASS|FAIL|DEFERRED
  OhosPlatformHandler: PASS|FAIL|DEFERRED
  ArkTsHostMethod: PASS|FAIL|DEFERRED
  ArkTsState: PASS|FAIL|DEFERRED
  ArkUiVisual: PASS|FAIL|DEFERRED
  ArkTsCallback: PASS|FAIL|DEFERRED
  RustAction: PASS|FAIL|DEFERRED
  MakepadStatusLabel: PASS|FAIL|DEFERRED
LayoutTrace:
  <native-id>: makepad_rect=<...> arkui_global=<...> arkui_size=<...> vp_px=<...>
CommandTrace:
  <native-id>: command=<...> revision=<...> queued=true|false result=<...>
```

If the simulator cannot satisfy clipboard or selection behavior, do not write
`Status: PASS`. Record the blocker and keep the status as deferred or failed.

## Risks

- ArkUI coordinate units may not match Makepad physical pixels.
- The overlay may not clip exactly like Makepad content.
- Simulator keyboard and pasteboard behavior can differ from real devices.
- OHOS target crates may fail to download in restricted network environments.
- DevEco signing/project metadata can be machine-local and should not be
  confused with source changes.
- Logs can be misleading if captured through a command that echoes its own grep
  pattern. Save raw logs first, then analyze them locally.
- Focus failures can originate in ArkUI focus, simulator keyboard behavior,
  Makepad command routing, or Rust callback handling. Treat them as separate
  layers during triage.
- ArkUI component creation may lag behind Rust platform ops. Early commands
  require queueing or explicit rejection.
- Boolean programmatic-update guards can be insufficient if ArkUI callback order
  differs from expectations. Prefer revision or event-count checks.
- Soft keyboard geometry should not be guessed. Use OHOS avoid-area or
  keyboard-height signals when keyboard layout becomes a requirement.
