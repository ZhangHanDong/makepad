spec: task
name: "AI Chat Liquid Glass Step 65 Native Fullscreen Probe"
tags: [makepad, liquid-glass, macos-native, fullscreen, studio]
---

## Intent

Add an opt-in Studio runnable that exercises the current macOS native fullscreen
fallback path from the app itself. This does not claim full native fullscreen
Liquid Glass support; it proves that the current explicit fallback can be
requested, observed, and restored through the normal Studio release run flow.

## Decisions

- Gate the probe behind `AICHAT_NATIVE_FULLSCREEN_PROBE=1`.
- Start the probe only after an Apple-native substrate resolves as installed.
- Request fullscreen through the normal Makepad `WindowRef::fullscreen` path.
- On fullscreen enter, rely on the existing native fallback transition, then
  request exit through `WindowRef::disable_fullscreen`.
- If the runtime does not emit a fullscreen-enter geometry event, log an
  explicit timeout instead of silently waiting forever.
- Add a dedicated Studio runnable:
  `makepad-example-aichat-macos-native-clear-fullscreen-probe`.
- Keep complete native fullscreen support marked incomplete because the current
  behavior is still a shader fallback while fullscreen.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `makepad.splash`
- `platform/src/os/apple/macos/macos.rs`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not change normal `macos-native-clear` behavior.
- Do not claim full native fullscreen glass is implemented.
- Do not use raw `cargo run` for UI validation; use Studio `RunItem`.

## Acceptance Criteria

### Scenario: fullscreen probe env parsing is explicit
Given `AICHAT_NATIVE_FULLSCREEN_PROBE` is set
When the value is `1` or `true`
Then the fullscreen probe is enabled
Test: `cargo test -p makepad-example-aichat aichat_native_fullscreen_probe_env_accepts_truthy_values --release`

### Scenario: fullscreen probe starts only for native installed substrates
Given native substrate resolution state changes
When the substrate is installed and native, or not installed
Then the probe starts only for the installed native case
Test: `cargo test -p makepad-example-aichat aichat_native_fullscreen_probe_starts_only_for_installed_native --release`

### Scenario: fullscreen probe reports an enter timeout
Given a runtime or Studio host does not emit a fullscreen-enter geometry event
When the probe waits too long after requesting fullscreen
Then it stops waiting and reports `native-fullscreen-probe=timeout phase=enter`
Level: unit helper plus Studio runtime log
Test: `cargo test -p makepad-example-aichat aichat_native_fullscreen_probe_enter_wait_has_timeout --release`

### Scenario: macOS handles Makepad fullscreen ops
Given `WindowRef::fullscreen` emits `CxOsOp::FullscreenWindow`
When macOS platform ops are dispatched
Then `FullscreenWindow` enters fullscreen and `NormalizeWindow` exits fullscreen
Test: `rg "CxOsOp::FullscreenWindow|CxOsOp::NormalizeWindow" platform/src/os/apple/macos/macos.rs`

### Scenario: Studio runnable exists
Given runtime validation must use Studio RunItem
When runnables are inspected
Then `makepad-example-aichat-macos-native-clear-fullscreen-probe` injects `AICHAT_NATIVE_FULLSCREEN_PROBE=1`
Test: `rg "macos-native-clear-fullscreen-probe|AICHAT_NATIVE_FULLSCREEN_PROBE|fullscreen_probe" makepad.splash`

### Scenario: completion audit keeps fullscreen support incomplete
Given this step probes fallback and restore rather than implementing native fullscreen
When completion audit status is inspected
Then fullscreen remains marked as an explicit fallback, not full native fullscreen support
Test: `rg "fullscreen.*explicit fallback|not full native fullscreen support|fullscreen fallback probe" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: aichat compiles
Given fullscreen probe state is wired into native startup and geometry handling
When aichat is checked
Then it compiles
Test: `cargo check -p makepad-example-aichat --release`
