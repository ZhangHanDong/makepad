# NativeTextInput POC Completion Audit

Objective: complete the staged NativeTextInput native host roadmap in order.

This audit records concrete evidence only. A stage is not closed by code alone
when the stage has an explicit runtime, Studio, simulator, device, or DevEco
acceptance gate.

## Stage Checklist

| Stage | Deliverable | Evidence | Status |
| --- | --- | --- | --- |
| P0A | macOS `NativeTextInput` compile-oriented skeleton | `widgets/src/native_text_input.rs`, `platform/src/os/apple/apple_native_text_input.rs`, `platform/src/cx_api.rs`; `cargo check -p makepad-widgets` passed | Closed |
| P0B | macOS runtime loop: AppKit callbacks, widget actions, `on_change`, focus/blur, selection, clipboard commands, cleanup, leak diagnostic path | Code exists in `widgets/src/native_text_input.rs`, `platform/src/os/apple/apple_native_text_input.rs`, `platform/src/action.rs`, `makepad.splash`; `tools/native_textinput_apple_static_check.sh` passed; native unit tests pass; standalone diagnostic leak evidence records live count 3 -> 0 | Runtime partially accepted; changed-event/manual visual checklist retained |
| P1 | shared typed native host registry with `NativeTextInput` and `NativeLabel` | `MacosNativeHost`, `NativeHostKind`, `NativeHostProps`, `NativeHostCommand`, `NativeLabel`; `tools/native_textinput_host_queue_static_check.sh` passed; platform tests cover TextInput and Label typed mount ops; user manually confirmed `Set Label` after UI fix | Code/static complete; visual checklist retained |
| P1.5 | clipped host/content composition | `apple_native_host::clipped_native_host_layout`, `apple_native_host::order_native_host_view`, `apple_webview` clip tests, `NativeTextInput` / `NativeLabel` clipped host views; `tools/native_textinput_host_queue_static_check.sh` passed | Code/static complete; scroll/z-order runtime checklist retained |
| P2 | iOS `UITextField` implementation | `apple_ios_native_text_input.rs`, iOS typed op handling; `tools/native_textinput_apple_static_check.sh` passed; `cargo check -p makepad-example-native-text-input --target aarch64-apple-ios` and `--release` passed | Compile/static complete, blocked on simulator/device runtime gate |
| P3 | Android `EditText` implementation | `MakepadActivity.java`, `MakepadNative.java`, `android_jni.rs`, `android.rs`; `tools/native_textinput_android_static_check.sh` passed; `cargo check -p makepad-widgets --target aarch64-linux-android` passed | Compile/static complete, blocked on cargo-makepad/device runtime gate |
| P4 | `NativeMountQueue` transaction layer | `NativeMountQueue` / `NativeMountMutation` in `cx_api.rs`, `Cx::flush_native_mount_queue`, per-platform flush calls; `tools/native_textinput_host_queue_static_check.sh` passed; platform tests cover coalescing, command barriers, close, and reentrant flush | Code/static complete; runtime checklist retained |
| P5 | OpenHarmony ArkUI leaf host | `open_harmony.rs`, `oh_callbacks.rs`, `makepad.ets`, `Index.ets`; ArkTS selection uses `TextInputController.setTextSelection`, clipboard uses `@kit.BasicServicesKit` pasteboard; simulator contract is `docs/native-textinput-ohos-simulator.spec.md` | Code present; simulator evidence, OHOS target compile, DevEco build, and device runtime validation not accepted |
| P6 | schema / codegen seed | `native_host_schema.rs`; `tools/native_textinput_schema_texture_static_check.sh` passed; schema/golden manifest tests pass | Seed complete, generator replacement not implemented |
| P7 | native texture layer seed | `native_texture_layer.rs`; `tools/native_textinput_schema_texture_static_check.sh` passed; texture layer unit tests pass | Seed complete, native widget texture rendering not implemented |

## Latest Local Verification

- `cargo check -p makepad-widgets` passed.
- `cargo check -p makepad-example-native-text-input --release` passed.
- `cargo check -p makepad-example-native-text-input --release --features makepad-widgets/diag-native-leak-count` passed.
- `cargo test -p makepad-example-native-text-input --test ui --no-run` passed;
  this compiles the example-level `makepad_test` selector suite without
  treating it as runtime evidence.
- `cargo check -p makepad-example-native-text-input --target aarch64-apple-ios` passed with pre-existing warnings.
- `cargo check -p makepad-example-native-text-input --target aarch64-apple-ios --release` passed with pre-existing warnings.
- `cargo check -p makepad-widgets --target aarch64-linux-android` passed with pre-existing warnings in `openxr.rs`.
- `cargo test -p makepad-widgets native_ --lib` passed, 9 tests.
- `cargo test -p makepad-platform native_ --lib` passed, 29 tests. This now
  includes command-barrier tests proving `NativeMountQueue` does not coalesce
  layout or props across same-host commands, a Label typed mount op test
  covering create/layout/text updates, and macOS native host clip tests.
- Targeted `rustfmt --check --config skip_children=true` over NativeHost-related Rust files passed.
- `git diff --check` passed.
- `tools/native_textinput_poc_verify.sh` passed in default mode after all
  current static gates were added; it is the repeatable wrapper for these
  non-UI gates and includes the iOS release target check.
- `tools/native_textinput_poc_verify.sh --skip-ios-release` also passed and is
  available for faster local iteration when the iOS release target check is not
  needed.
- `tools/native_textinput_static_evidence.sh --skip-ohos-target` passed and
  regenerated `docs/native-textinput-evidence/non-ui-static.md` with
  `AndroidTarget: checked`, `IosRelease: checked`, and `OhosTarget: skipped`.
  This is local-only evidence; the completion audit still requires a real
  `OhosTarget: checked` run.
- `tools/native_textinput_runtime_preflight.sh` is covered by `bash -n` in the
  non-UI verifier. Live execution is expected to fail in this sandbox for the
  same Studio bridge connection reason listed below, but it should be run before
  `tools/native_textinput_completion_run_all.sh` in the runtime-capable
  environment. The static evidence marker now records this as
  `runtime_preflight`.
- `tools/native_textinput_example_static_check.sh` passed and is included in
  the non-UI verification wrapper to assert workspace membership, example
  package naming, required smoke controls/actions, Studio RunItem registration,
  diagnostic feature wiring, Studio clear-build guard defaults, and roadmap
  references to the canonical repository examples used for runtime validation.
  It also asserts that `examples/native_text_input/tests/ui.rs` covers the
  canonical control IDs used by the smoke app, that runtime helper transcripts
  and screenshot markers are audited, that helper startup waits are build-id
  scoped, that duplicate `AppStarted` / `RunViewCreated` waits are avoided, and
  that `completion_run_all` carries each generated `BuildId:` into the next
  helper's `--clear-build-id`.
- `tools/native_textinput_studio_gate_static_check.sh` passed and is included
  in the non-UI verification wrapper to assert normal RunItem JSONL, diagnostic
  RunItem JSONL, `ClearBuild` packet shape, and refusal of unguarded `RunItem`.
- `tools/native_textinput_host_queue_static_check.sh` passed and is included
  in the non-UI verification wrapper to assert typed native host enums, native
  mount queue coalescing/close/order/command-barrier/reentrant tests, widget-facing
  `CxNativeTextInput` / `CxNativeLabel` APIs, `NativeLabel` registration,
  macOS/iOS native host registries, clipped host layout, macOS native host
  z-order helper wiring, and per-platform native queue flush calls.
- `tools/native_textinput_apple_static_check.sh` passed and is included in the
  non-UI verification wrapper to assert macOS AppKit delegate wiring,
  macOS retain/release cleanup, diagnostic leak feature wiring, iOS
  target/action wiring, programmatic guards, selection callbacks, clipboard
  commands, typed command dispatch, and widget Drop close requests.
- `tools/native_textinput_android_static_check.sh` passed and is included in
  the non-UI verification wrapper to assert local Android crate patches, Java
  `EditText` overlay registry, programmatic guard, changed/focus/selection
  callbacks, native command methods, JNI callback posting, and Rust command
  dispatch.
- `tools/native_textinput_ohos_static_check.sh` passed and is included in the
  non-UI verification wrapper to assert ArkTS controller, pasteboard,
  permission, and callback wiring.
- `tools/native_textinput_schema_texture_static_check.sh` passed and is
  included in the non-UI verification wrapper to assert NativeHost schema
  exports, manifest golden coverage, schema-vs-typed-contract tests, texture
  layer bridge exports, `VideoExternal` allocation, frame generation tests, and
  the documented P6/P7 seed boundaries.
- `tools/native_textinput_studio_gate.sh --print-jsonl --list-builds` and
  `tools/native_textinput_studio_gate.sh --print-jsonl --clear-build-id 6`
  produced the expected Studio bridge JSONL. Live bridge execution is still
  blocked by the sandbox connection error below.
- `bash -n tools/native_textinput_macos_studio_smoke.sh
  tools/native_textinput_macos_leak_smoke.sh
  tools/native_textinput_macos_composition_evidence.sh
  tools/native_textinput_device_runtime_evidence.sh
  tools/native_textinput_static_evidence.sh
  tools/native_textinput_completion_run_all.sh
  tools/native_textinput_completion_audit.sh` passed via the non-UI verifier.
- `tools/native_textinput_studio_gate.sh --print-jsonl` correctly refused to
  launch a RunItem without `--clear-build-id` or `--allow-no-clear`.

## Blocking Evidence

- Studio bridge FIFO helpers can still hang in this Codex tool environment even
  though the same Studio protocol works through a persistent TTY bridge. Keep
  helper self-tests/static validation in the agent loop, and use
  `docs/native-textinput-evidence/macos-manual-checklist.md` for any remaining
  human visual checks.
- Running `cargo test -p makepad-example-native-text-input --test ui
  -- --test-threads=1` in this sandbox fails before app startup because
  `makepad_test` cannot bind its local headless hub server
  (`failed to bind http server at 127.0.0.1:0`). The suite is therefore
  compiled here with `--no-run`; execution belongs with the runtime-capable
  local/CI environment and still does not replace Studio RunItem evidence.
- Android plain bin target check is not the real app build path:
  `cargo check -p makepad-example-native-text-input --target aarch64-linux-android`
  fails because `app_main!` intentionally does not emit a bin `main` on
  Android; cargo-makepad builds Android apps as `--lib --crate-type=cdylib`.
  The library graph target check now passes, but cargo-makepad/device runtime
  validation is still required.
- OpenHarmony target check cannot reach crates.io in this sandbox. Latest
  attempted command: `cargo check -p makepad-platform --target
  aarch64-unknown-linux-ohos`; recent failing downloads include
  `ohos-sys 0.2.2`, `napi-ohos 0.1.3`, and
  `napi-derive-backend-ohos 0.0.7` from `static.crates.io`. The OHOS-only
  dependencies in `platform/Cargo.toml` are pinned to the current `Cargo.lock`
  versions to keep this target gate reproducible once crates can be fetched.
- OpenHarmony `selectAllNativeTextInput`, `copyNativeTextInput`,
  `cutNativeTextInput`, and `pasteNativeTextInput` now have ArkTS
  selection/clipboard implementations, but still need DevEco/device validation.
- OpenHarmony simulator acceptance is now tracked by
  `docs/native-textinput-ohos-simulator.spec.md` and
  `docs/native-textinput-ohos-simulator.plan.md`. That gate requires layout
  trace, command trace, raw hilog evidence, screenshot evidence, callback-order
  notes, revision/event-count guard validation, and explicit handling of
  commands that arrive before ArkUI component attachment.

## Not Yet Accepted

- macOS manual changed-event confirmation from real typing in the standalone
  AppKit window.
- macOS scroll/clipping/z-order visual confirmation for the clipped native host
  panel.
- iOS simulator/device input, IME, emoji, selection, and clipboard behavior.
- Android cargo-makepad package/deploy, input, IME, emoji, selection, and
  clipboard behavior.
- OpenHarmony simulator runtime evidence for layout trace, command trace,
  focus/blur controller behavior, early command handling, set-text guard,
  selection/cut/paste callback ordering, and screenshot/log correlation.
- OpenHarmony DevEco compile and device runtime validation for selection,
  clipboard, permission, callback behavior, and soft keyboard behavior.
- Any claim that P6 generated code replaces the current hand-written typed
  enums.
- Any claim that P7 renders native widgets into Makepad texture/shader content.

## Prompt-to-Artifact Checklist

The objective is "complete the eight stages in order." The concrete artifact
mapping below is the acceptance source of truth for closing, not just a
progress summary.

| Requirement | Required evidence | Current evidence | Coverage judgment |
| --- | --- | --- | --- |
| P0B macOS runtime loop: visible native field, focus/blur, user input changed action, programmatic set_text no changed loop, selection, clipboard, cleanup | Studio `RunItem` screenshot/click/type evidence, native actions in widget state, leak counter or `leaks` evidence | `tools/native_textinput_apple_static_check.sh`, widget/platform unit tests, diagnostic RunItem wiring, `docs/native-textinput-evidence/macos-leak-runtime.md`, manual checklist | Code/static and leak covered; changed-event manual checklist retained |
| P1 shared host registry: `NativeTextInput` and `NativeLabel` share typed host lifecycle | Static contract plus Studio run showing both components update through the shared registry | `tools/native_textinput_host_queue_static_check.sh`, `examples/native_text_input`, native tests, user manual `Set Label` validation | Code/static covered; visual checklist retained |
| P1.5 clipped hybrid composition: host/content two-layer clipping, scroll/z-order behavior | Studio runtime scenario with clipped or scrolled native widgets and screenshots/widget dumps | `apple_native_host` tests, `apple_webview` clip tests, `tools/native_textinput_host_queue_static_check.sh` | Unit/static covered; scroll/z-order runtime not accepted |
| P2 iOS `UITextField`: focus/keyboard/input/emoji/selection/clipboard | Simulator or device run of `makepad-example-native-text-input` | `tools/native_textinput_apple_static_check.sh`, iOS dev/release target checks | Compile/static covered; simulator/device not accepted |
| P3 Android `EditText`: JNI bridge, IME/input/emoji/selection/clipboard | Android cargo-makepad package/deploy and device/emulator run | `tools/native_textinput_android_static_check.sh`, `cargo check -p makepad-widgets --target aarch64-linux-android` | Compile/static covered; package/device not accepted |
| P4 `NativeMountQueue`: frame-boundary native mutations, coalescing, command order, close behavior, per-platform flush | Unit tests plus visual runtime check for multi-native-view same-frame updates | `tools/native_textinput_host_queue_static_check.sh`, platform queue tests | Unit/static covered; Studio visual runtime not accepted |
| P5 OpenHarmony ArkUI leaf host: create/update/detach/focus/blur/changed/selection/clipboard | OHOS target compile, DevEco ArkTS compile, simulator evidence, device run | `tools/native_textinput_ohos_static_check.sh`; simulator contract in `docs/native-textinput-ohos-simulator.spec.md`; target check blocked by crates.io DNS | Static covered; simulator/compile/device not accepted |
| P6 schema/codegen seed | Stable manifest and parity tests against typed Rust contract; explicit no-claim that generated code replaces handwritten enums | `native_host_schema.rs`, manifest golden, `cx_api` parity tests, `tools/native_textinput_schema_texture_static_check.sh` | Seed accepted; generator replacement intentionally out of scope |
| P7 native texture layer seed | External texture allocation, host binding, frame generation contract; explicit no-claim that native widgets render into textures | `native_texture_layer.rs` tests, `tools/native_textinput_schema_texture_static_check.sh` | Seed accepted; native widget texture rendering intentionally out of scope |

Proxy-signal rule: `cargo check`, static scripts, manifest tests, and unit tests
can close compile/static/seed requirements only. They do not close any row whose
required evidence names Studio, simulator, DevEco, device, screenshot, click,
type, leak counter, or visual behavior.

The example-level `makepad_test` suite is treated the same way: its `--no-run`
compile check only locks selector coverage for the canonical smoke app, and a
headless `cargo test ... --test ui` run cannot close runtime/device gates. The
completion audit rejects runtime evidence files that cite `cargo test ... --test
ui` or `makepad_test` as their command source.

Runtime evidence is tracked under `docs/native-textinput-evidence/` and checked
by `tools/native_textinput_completion_audit.sh`. The canonical app for those
checks is the checked-in `examples/native_text_input` example, launched through
Studio RunItem `makepad-example-native-text-input`,
`makepad-example-native-text-input-macos-standalone`, or the diagnostic
`makepad-example-native-text-input-macos-standalone-diag` RunItem. Mobile/device runtime evidence
uses the same example through `makepad-example-native-text-input-ios-sim`,
`makepad-example-native-text-input-android`, and
`makepad-example-native-text-input-ohos`.
