spec: task
name: "AI Chat Liquid Glass Step 66 Native Display Frame Snapshot"
tags: [makepad, liquid-glass, apple-native, multi-display, backing-scale]
---

## Intent

Strengthen the multi-display validation probe by logging native panel frame
snapshots when a macOS native-glass window reports a backing-scale change. Step
54 only recorded `dpi_factor` and window position; this step records the
Makepad logical rect and the AppKit frame used for each native glass panel.

## Decisions

- Keep descriptor geometry in Makepad logical units.
- Convert to AppKit frames only at the macOS backend boundary.
- Log snapshots from the macOS backend because it owns the final native frame
  conversion and the cached native batch.
- Treat the snapshot as validation evidence only; multi-display support remains
  unproven until a real display-move run is reviewed.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_window.rs`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not add physical-pixel scaling to native AppKit frames.
- Do not expose AppKit frame types in shared descriptor structs.
- Do not claim multi-display support is complete.

## Acceptance Criteria

### Scenario: panel frame snapshot lines are deterministic
Given a native glass panel descriptor and its AppKit frame
When the platform log line is formatted
Then it includes container id, panel id, Makepad rect, AppKit frame, z-order, and visible state
Test: `cargo test -p makepad-platform native_glass_panel_frame_snapshot_line_contains_logical_and_appkit_frames --release`

### Scenario: backing-scale changes trigger native frame snapshots
Given a macOS window geometry change changes `dpi_factor`
When a native glass batch is cached for that window
Then the macOS backend logs `native-display-frame-snapshot` and `native-panel-frame`
Test: `rg "native-display-frame-snapshot|native-panel-frame|log_native_glass_frame_snapshot" platform/src/os/apple/macos`

### Scenario: display probe docs remain conservative
Given multi-display remains a Phase H gate
When the advanced behavior gates and completion audit are inspected
Then they mention frame snapshots and still mark multi-display as unproven
Test: `rg "native-display-frame-snapshot|multi-display.*unproven|frame snapshot" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: platform compiles
Given native frame snapshot logging is wired into macOS geometry changes
When platform is checked
Then it compiles
Test: `cargo check -p makepad-platform --release`
