spec: task
name: "AI Chat Liquid Glass Step 54 Native Display Change Probe"
tags: [makepad, liquid-glass, apple-native, multi-display, backing-scale]
---

## Intent

Add a validation probe for multi-display and backing-scale changes while the
macOS native underlay is active. The current AppKit backend uses logical point
coordinates for native views, so the next gate is to record when the window
moves across displays with a different `dpi_factor`.

## Decisions

- Treat multi-display support as unproven until a native-glass run records
  backing-scale changes and panel alignment is manually checked.
- Log backing-scale changes only while the effective aichat appearance is native.
- Keep the probe in aichat so it does not change platform descriptor semantics.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not claim multi-display support is complete.
- Do not change native descriptor coordinates from Makepad logical units.
- Do not add physical-pixel scaling to native AppKit frames.

## Acceptance Criteria

### Scenario: backing scale changes are detected
Given the old and new window dpi factors differ
When the backing-scale helper is evaluated
Then it reports a display scale change
Test: `cargo test -p makepad-example-aichat aichat_native_display_change_probe_detects_dpi_change --release`

### Scenario: subpixel dpi jitter is ignored
Given the old and new window dpi factors differ only by tiny floating point jitter
When the backing-scale helper is evaluated
Then it does not report a display scale change
Test: `cargo test -p makepad-example-aichat aichat_native_display_change_probe_ignores_jitter --release`

### Scenario: display probe is documented
Given multi-display remains a Phase H gate
When the advanced behavior gates and completion audit are inspected
Then they reference the native display/backing-scale probe and still say support is unproven
Test: `rg "native-display-change|backing-scale|multi-display.*unproven" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
