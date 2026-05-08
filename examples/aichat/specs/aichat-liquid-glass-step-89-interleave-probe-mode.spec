spec: task
name: "AI Chat Liquid Glass Step 89 Interleave Probe Mode"
tags: [makepad, liquid-glass, apple-native, interleave, macos, probe]
---

## Intent

Convert the hidden macOS interleave probe flag from a boolean into a typed
probe mode so future LowerScene proof rendering can be added without overloading
truthy environment values.

## Decisions

- `1`, `true`, and `on` continue to mean lifecycle/drawable probing.
- `lower-scene-clear` is reserved as the first explicit LowerScene proof mode.
- The reserved proof mode must not yet render Makepad UI or expose
  `AppleNativeInterleave`.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not change the existing Studio runnable value.
- Do not implement LowerScene rendering in this step.
- Do not change primary draw-pass routing.

## Acceptance Criteria

### Scenario: typed probe mode exists
Given future probes need more than boolean state
When the macOS source is inspected
Then it defines `MacosInterleaveProbeMode` with `LifecycleDrawable` and `LowerSceneClear`
Test: `rg "enum MacosInterleaveProbeMode|LifecycleDrawable|LowerSceneClear" platform/src/os/apple/macos/macos.rs`

### Scenario: truthy values keep lifecycle drawable behavior
Given the existing Studio runnable injects `1`
When env parsing is inspected
Then `1`, `true`, and `on` map to `LifecycleDrawable`
Test: `rg 'Some\\("1"\\)|Some\\("true"\\)|Some\\("on"\\)|MacosInterleaveProbeMode::LifecycleDrawable' platform/src/os/apple/macos/macos.rs`

### Scenario: lower scene clear mode is reserved
Given LowerScene proof rendering is not implemented yet
When env parsing is inspected
Then `lower-scene-clear` maps to `MacosInterleaveProbeMode::LowerSceneClear`
Test: `rg 'lower-scene-clear|MacosInterleaveProbeMode::LowerSceneClear' platform/src/os/apple/macos/macos.rs`

### Scenario: probe layer stores the selected mode
Given runtime logs and later drawing need the selected mode
When `MacosInterleaveProbeLayer` is inspected
Then it stores `mode: MacosInterleaveProbeMode`
Test: `rg "mode: MacosInterleaveProbeMode" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given this is a probe-mode refactor
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only refactors macOS probe mode
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-89-interleave-probe-mode.spec platform/src/os/apple/macos/macos.rs "`
