spec: task
name: "AI Chat Liquid Glass Step 67 Native Geometry Frame Snapshot"
tags: [makepad, liquid-glass, apple-native, stage-manager, split-view]
---

## Intent

Add an opt-in macOS native-glass geometry snapshot gate for Stage Manager and
split-view style window movement. Step 66 logs frame snapshots when backing
scale changes; this step lets validation runs also log snapshots for same-screen
position and size changes without making normal native runs noisy.

## Decisions

- Gate same-screen geometry snapshots behind
  `MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT=1`.
- Treat position, inner-size, or backing-scale changes as geometry changes.
- Keep backing-scale snapshots always enabled because they are the multi-display
  evidence path from Step 66.
- Do not claim Stage Manager or split-view support is complete.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_window.rs`
- `makepad.splash`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not log same-screen geometry snapshots unless the env gate is enabled.
- Do not change native glass descriptor coordinates.
- Do not claim Stage Manager or split-view validation is complete without a real
  runtime smoke run.

## Acceptance Criteria

### Scenario: geometry snapshot env parsing is explicit
Given `MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT` is set
When the value is `1` or `true`
Then same-screen geometry snapshots are enabled
Test: `cargo test -p makepad-platform native_glass_geometry_snapshot_env_accepts_truthy_values --release`

### Scenario: geometry changes are detected
Given old and new window geometry values
When position, inner-size, or backing-scale changes
Then geometry change detection reports true; identical geometry reports false
Test: `cargo test -p makepad-platform native_glass_window_geometry_changed_detects_position_size_and_dpi --release`

### Scenario: Studio runnable exists
Given Stage Manager-style validation must use Studio RunItem
When runnables are inspected
Then `makepad-example-aichat-macos-native-clear-geometry-probe` injects `MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT=1`
Test: `rg "macos-native-clear-geometry-probe|MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT|geometry_probe" makepad.splash`

### Scenario: docs remain conservative
Given Stage Manager and split-view remain Phase H gates
When advanced behavior gates and completion audit are inspected
Then they mention the geometry snapshot probe and still mark support as unproven
Test: `rg "geometry snapshot|Stage Manager.*unproven|split-view" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: platform compiles
Given geometry snapshot logging is wired into macOS geometry changes
When platform is checked
Then it compiles
Test: `cargo check -p makepad-platform --release`
