spec: task
name: "AI Chat Liquid Glass Step 95 LowerScene Role Runtime Probe"
tags: [makepad, liquid-glass, apple-native, interleave, macos, runtime-probe]
---

## Intent

Add a dedicated macOS runtime probe mode that forces the current window pass
through the role-based LowerScene route. This validates the Step 94
`lower-scene-role=draw` path in Studio before aichat is refactored into
separate LowerScene and UpperUi passes.

## Decisions

- `lower-scene-role` is diagnostic only.
- It installs the same sibling LowerScene surface as the clear/mirror probes.
- It forces the window pass to `MacosMetalSurfaceRole::LowerScene` inside macOS
  routing without changing normal `Primary` defaults.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- `makepad.splash`
- this step spec

### Constraints

- Do not wire aichat production UI to LowerScene yet.
- Do not expose `apple-native-interleave` as an enabled production backend.
- Do not change normal `macos-native-clear` behavior.

## Acceptance Criteria

### Scenario: lower-scene-role probe mode exists
Given Step 94 needs runtime validation
When macOS probe mode parsing is inspected
Then `lower-scene-role` maps to `MacosInterleaveProbeMode::LowerSceneRole`
Test: `rg "lower-scene-role|LowerSceneRole" platform/src/os/apple/macos/macos.rs`

### Scenario: role probe forces LowerScene routing
Given the role probe is active
When macOS surface role selection is inspected
Then `forces_lower_scene_role` makes `macos_surface_role_for_window_pass` return `MacosMetalSurfaceRole::LowerScene`
Test: `rg "forces_lower_scene_role|MacosInterleaveProbeMode::LowerSceneRole|MacosMetalSurfaceRole::LowerScene" platform/src/os/apple/macos/macos.rs`

### Scenario: Studio has a dedicated role probe runnable
Given runtime validation must use Studio RunItem
When Studio runnables are inspected
Then a dedicated runnable injects `lower-scene-role`
Test: `rg "macos-native-clear-interleave-lower-scene-role|lower-scene-role" makepad.splash`

### Scenario: platform crate still compiles
Given this changes macOS routing mode selection
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: aichat still compiles
Given the probe runnable still launches aichat
When aichat is checked
Then script/app compilation succeeds
Test: `cargo check -p makepad-example-aichat --release`

### Scenario: no unrelated files are changed
Given this step only adds a role routing runtime probe
When the worktree status is inspected
Then only the allowed files and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-95-lower-scene-role-runtime-probe.spec makepad.splash platform/src/os/apple/macos/macos.rs "`
