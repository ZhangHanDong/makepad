# OHOS NativeTextInput Simulator Landing Plan

Status: planning document for the OpenHarmony local simulator slice. This is
not a final P5 closeout. The goal is to make the existing ArkTS overlay host
implementation build and run on a local OHOS simulator, then capture enough
runtime evidence to guide the next code fixes.

Canonical contract: `docs/native-textinput-ohos-simulator.spec.md`.

## Goal

Bring up `examples/native_text_input` on a local OpenHarmony simulator and
validate the first ArkUI `TextInput` wrap path:

```text
NativeTextInput widget
  -> CxNativeTextInput typed NativeMountQueue ops
  -> CxOsOp::{Create,Update,Command,Detach,Close}NativeView
  -> platform/src/os/linux/open_harmony/open_harmony.rs
  -> ArkTS ArkGlue
  -> Index.ets Stack overlay of ArkUI TextInput above Makepad XComponent
  -> ArkTS callbacks
  -> oh_callbacks.rs
  -> NativeTextInputChanged / FocusChanged / SelectionChanged actions
```

## Non-Goals

- Do not close the full P5 OpenHarmony stage yet.
- Do not implement a pure ArkUI Native Node host in this slice.
- Do not add a new ad hoc OHOS app; use `examples/native_text_input`.
- Do not validate real-device-only behavior unless a device is available.
- Do not include iOS, Android, or macOS work in this task.
- Do not import Westlake adapter techniques such as smali patches, boot image
  regeneration, LD_PRELOAD shims, AOSP InputChannel bridges, BPF grants, or TLS
  workarounds. Those belong to Android compatibility runtime bring-up, not this
  ArkUI overlay implementation.

## Westlake-Derived Validation Discipline

The `westlake-noice-ohos` repository is useful as a validation reference, not as
an implementation template. It shows that OHOS runtime problems are easy to
misclassify if a test only checks the final screen state. This simulator slice
must therefore keep the native text input path split into explicit observation
points.

Required observation chain:

```text
Rust widget command
  -> CxOsOp native view op
  -> OHOS platform handler
  -> NAPI call into ArkGlue / Index.ets
  -> ArkTS nativeTextInputs state mutation
  -> ArkUI TextInput visible/focused/edited state
  -> ArkTS callback
  -> oh_callbacks.rs
  -> Rust NativeTextInput action
  -> visible Makepad status label
```

When a runtime check fails, identify the first broken link in that chain before
changing code. For example, if tapping does not focus the input, first prove
whether the ArkUI `TextInput` itself can focus and whether
`focusControl.requestFocus(...)` fires before assuming the Makepad widget or Rust
action path is wrong.

Westlake also showed that synthetic probes are not enough. A passing command or
log marker does not close the runtime check unless it is tied to a real visible
effect or a callback observed at the next layer.

## RNOH-Derived ArkUI Lessons

The older `rnoh` repository is useful as an ArkUI wrapper reference, not as an
architecture to copy. Its React Native, Fabric, JS bundle, TurboModule, and
component descriptor layers are intentionally out of scope for Makepad
`NativeTextInput`. The useful parts are the lower-level ArkUI observations:

- `RNSurface.ets` records `onAreaChange` width, height, global position, and
  `vp2px(1)`. The OHOS simulator evidence must therefore log the Makepad rect,
  ArkUI global position, and vp/px ratio for each native text input.
- `RNTextInput.ets` keeps a controller per input and uses
  `TextInputController.stopEditing()` / `TextAreaController.stopEditing()` for
  blur. If `focusControl.requestFocus(...)` alone is unreliable, validate the
  controller blur path before changing Rust widget semantics.
- RNOH uses a hidden focus sink to work around ArkUI focus behavior. If blur
  does not visibly remove focus, add a minimal focus-sink experiment to the
  OHOS host and record it in evidence.
- Text change, selection, paste, and cut callbacks have ordering hazards.
  Programmatic `set_text` should use a revision or event-count guard rather
  than relying only on a boolean "programmatic update" flag.
- `RNComponentCommandHub.ts` backlogs commands that arrive before a component is
  registered. The OHOS host should either queue `focus`, `blur`, and `set_text`
  by native input id until the ArkUI component exists, or explicitly log and
  reject the stale command.
- Keyboard handling should prefer window avoid-area or keyboard-height signals
  when soft keyboard layout becomes part of the test scope.
- DevEco package and symlink behavior is fragile in older RNOH notes. Keep the
  generated OHOS project self-contained and avoid making the simulator test
  depend on project-external local packages.

These lessons refine the current ArkTS overlay path. They do not justify
introducing a React Native style bridge or a broad descriptor system for the
simulator slice.

## Phase 0: Environment Baseline

Install or verify:

- DevEco Studio or DevEco command-line tools.
- OpenHarmony SDK that matches the local simulator image.
- `hdc` in `PATH`.
- Rust OHOS target/toolchain support through Makepad tooling.

Commands:

```bash
echo "$DEVECO_HOME"
which hdc
hdc list targets
cargo makepad ohos install-toolchain
```

Verified local macOS setup:

```bash
export DEVECO_HOME=/Applications/DevEco-Studio.app/Contents
export JAVA_HOME=/Applications/DevEco-Studio.app/Contents/jbr/Contents/Home
export PATH="$JAVA_HOME/bin:$DEVECO_HOME/sdk/default/openharmony/toolchains:$DEVECO_HOME/tools/ohpm/bin:$DEVECO_HOME/tools/node/bin:$PATH"
export MAKEPAD=ohos_sim
```

On this machine, the Rust target `aarch64-unknown-linux-ohos` is installed and
the release HAP build succeeds. `hdc list targets` currently reports `[Empty]`,
so the simulator still has to be created and started in DevEco before install
and runtime validation can proceed.

Expected:

- `DEVECO_HOME` points at the installed DevEco command-line tools or Studio
  SDK location.
- `hdc list targets` can see the local simulator after it starts.
- `cargo makepad ohos install-toolchain` completes without reinstall errors.

If the local simulator is x86_64, set:

```bash
export MAKEPAD=ohos_sim
```

Keep this variable in the shell used for build/run commands.

## Phase 1: Static Contract Check

Run the existing static guard:

```bash
tools/native_textinput_ohos_static_check.sh
```

This confirms the repository still has:

- ArkTS `TextInputController`.
- ArkTS `TextInput` overlay binding.
- selection callback wiring.
- pasteboard usage and permission.
- pinned OHOS NAPI dependencies.

Then run target checks as the local SDK allows:

```bash
cargo check -p makepad-platform --target aarch64-unknown-linux-ohos
```

For an x86_64 simulator:

```bash
cargo check -p makepad-platform --target x86_64-unknown-linux-ohos
```

If crates cannot be fetched, record the failing crate and do not treat the
runtime task as closed.

## Phase 2: DevEco Project Generation

Generate the DevEco project from the checked-in example:

```bash
MAKEPAD=ohos_sim cargo makepad ohos \
  --deveco-home="$DEVECO_HOME" \
  deveco -p makepad-example-native-text-input --release
```

Expected output location:

```text
target/makepad-open-harmony/makepad_example_native_text_input
```

Open this generated project in DevEco Studio if signing or simulator setup is
not already configured.

## Phase 3: Simulator Build and Run

Run through Makepad tooling:

```bash
MAKEPAD=ohos_sim cargo makepad ohos \
  --deveco-home="$DEVECO_HOME" \
  run -p makepad-example-native-text-input --release
```

Studio RunItem equivalent:

```text
makepad-example-native-text-input-ohos
```

For formal UI/runtime validation, prefer the Studio RunItem path and capture
the build id/log transcript. Direct command-line runs are acceptable only for
initial environment bring-up and failure triage.

Useful diagnostics:

```bash
hdc list targets
hdc hilog
cargo makepad ohos --deveco-home="$DEVECO_HOME" hilog -p makepad-example-native-text-input
```

Do not run inline `hilog | grep` commands as the source of truth. Some OHOS/hdc
setups echo the command text into the same stream that is being searched, which
can create false positives. Capture first, analyze locally:

```bash
hdc shell "hilog -x > /data/local/tmp/makepad_native_textinput.log"
hdc file recv /data/local/tmp/makepad_native_textinput.log \
  docs/native-textinput-evidence/ohos-runtime.log
rg "NativeTextInput|ArkGlue|focus|selection|paste" \
  docs/native-textinput-evidence/ohos-runtime.log
```

## Phase 4: First Runtime Checklist

On the local simulator, validate:

- The HAP installed in the simulator is the HAP just built for this run.
- App launches and Makepad content appears inside the `XComponent`.
- Primary ArkUI `TextInput` appears at the Makepad `native_input` location.
- Secondary ArkUI `TextInput` appears at the `secondary_native_input` location.
- Each input logs Makepad rect, ArkUI global position, width, height, and
  vp/px ratio during layout.
- Tapping a native input gives focus and opens the soft keyboard if the
  simulator supports it.
- Manual ASCII input changes the field text.
- Manual input posts `NativeTextInputChanged` and updates the Makepad status
  labels.
- `Primary Set` and `Secondary Set` update the native fields without producing
  an extra changed loop or overwriting newer manual input.
- focus / blur commands work for both fields.
- select all / copy / cut / paste work at least for plain text.
- selection changes post `NativeTextInputSelectionChanged`.
- `Set Both` updates both inputs in one interaction.
- Close/relaunch does not leave stale ArkUI `TextInput` overlays.

Record any simulator limitations separately from implementation bugs. IME,
emoji, and system clipboard behavior may differ between simulator and device.

For each failed checklist item, record the deepest layer reached:

- `Rust op emitted`
- `OHOS platform handler reached`
- `ArkTS host method reached`
- `ArkTS state changed`
- `ArkUI visual changed`
- `ArkTS callback fired`
- `Rust action received`
- `Makepad status label changed`

This keeps failures actionable and prevents overfitting fixes to the wrong
layer.

## Phase 5: Evidence Capture

When the simulator run is useful enough to preserve, write:

```text
docs/native-textinput-evidence/ohos-runtime.md
docs/native-textinput-evidence/ohos-runtime.log
```

Helper shape:

```bash
tools/native_textinput_device_runtime_evidence.sh \
  --platform ohos \
  --clear-build-id <previous-build-id> \
  --device "OpenHarmony local simulator (<hdc target>)" \
  --verified "DevEco changed selection clipboard"
```

For an initial simulator-only pass, the evidence may explicitly say
`Device: OpenHarmony local simulator (...)`. Do not mark full P5 closed until
the completion audit requirements are satisfied.

Minimum evidence contents:

- `hdc list targets` output.
- DevEco / JBR / SDK paths used for the run.
- command transcript for project generation, build, install, and launch.
- HAP path, byte size, and checksum.
- `libmakepad.so` path, byte size, and checksum when available.
- raw `hilog` capture, saved before local filtering.
- `snapshot_display` screenshot or Studio screenshot from the running app.
- per-input layout trace: native id, Makepad rect, ArkUI global position,
  applied width/height, and vp/px ratio.
- per-command trace: native id, command, command revision or event count,
  whether it ran immediately or was queued before ArkUI creation.
- manual checklist results with PASS / FAIL / DEFERRED.

The screenshot is part of the evidence, not decoration. If the log says a field
was created but the screenshot does not show it at the expected location, the
result is not a pass.

## Phase 6: Likely Fix Areas

Expect work in these files:

- `tools/open_harmony/deveco/entry/src/main/ets/pages/Index.ets`
- `tools/open_harmony/deveco/entry/src/main/ets/makepad/makepad.ets`
- `platform/src/os/linux/open_harmony/open_harmony.rs`
- `platform/src/os/linux/open_harmony/oh_callbacks.rs`
- `tools/native_textinput_ohos_static_check.sh`
- `docs/native-textinput-evidence/README.md`

Likely risks:

- coordinate units: Makepad pixels vs ArkUI vp/px.
- z-order and clipping of ArkUI overlay above the `XComponent`.
- simulator keyboard behavior.
- pasteboard permission and user authorization.
- programmatic update guard around `TextInput.onChange`; prefer command
  revision or event count if boolean guards prove ambiguous.
- Rust-to-ArkTS calls before `ArkGlue.nativeTextInputHost` or the target ArkUI
  component is ready; queue or explicitly reject with an evidence log.
- ArkUI focus and blur reliability; validate `TextInputController.stopEditing()`
  and, if needed, a hidden focus sink before changing the shared widget API.
- ArkUI callback ordering for `onChange`, selection, paste, and cut.
- soft keyboard layout; prefer window avoid-area or keyboard-height signals
  instead of guessing keyboard height.
- measurement ambiguity: command output, logs, and screen state can disagree.
  Treat the screen plus the next-layer callback as the decisive signal.
- focus/input routing ambiguity: simulator touch, soft keyboard, ArkUI focus,
  and Rust actions are separate layers and must be isolated when debugging.

## Done for This Slice

This simulator slice is complete when:

- static OHOS guard passes;
- the example builds and installs to the local simulator;
- at least one native text input can be focused and edited;
- changed/focus/selection callbacks reach Rust;
- set/select/clipboard commands are either passing or documented as simulator
  limitations with concrete logs;
- `ohos-runtime.md` records the command, target, build id, and transcript.
