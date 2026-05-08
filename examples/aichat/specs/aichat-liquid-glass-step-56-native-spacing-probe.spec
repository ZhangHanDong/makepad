spec: task
name: "AI Chat Liquid Glass Step 56 Native Spacing Probe"
tags: [makepad, liquid-glass, apple-native, spacing, morph, phase-h]
---

## Intent

Strengthen the validation path for Apple native container spacing. The macOS
backend already maps `GlassContainer.spacing` to `setSpacing:`, but animated
morph transitions should not be claimed until spacing changes are proven to
invalidate the native batch and runtime logs expose the applied spacing.

## Decisions

- Treat static `GlassContainer.spacing` mapping as implemented.
- Treat animated spacing/morph transitions as unproven Phase H work.
- Add explicit test and log evidence for spacing changes before implementing
  animation.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos_window.rs`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not add animated spacing behavior.
- Do not change the `GlassContainer.spacing` descriptor field shape.
- Do not change panel frame conversion.

## Acceptance Criteria

### Scenario: spacing changes invalidate native batch equivalence
Given two native glass batches differ only in container spacing
When native batch equivalence is evaluated
Then the batches are not equivalent
Test: `cargo test -p makepad-platform native_glass_batch_equivalent_detects_spacing_change --release`

### Scenario: native spacing application is logged
Given macOS applies native container spacing
When the macOS backend source is inspected
Then it logs `native-container-spacing` with the container id and spacing value
Test: `rg "native-container-spacing|setSpacing" platform/src/os/apple/macos/macos_window.rs`

### Scenario: descriptor shape remains stable
Given this step must not change the shared descriptor shape
When the descriptor source is inspected
Then `NativeGlassContainerDescriptor` still exposes `spacing: f64`
Test: `rg "pub struct NativeGlassContainerDescriptor|pub spacing: f64" platform/src/event/window.rs`

### Scenario: panel frame conversion remains unchanged
Given this step must not alter panel geometry conversion
When the macOS backend source is inspected
Then panels are still converted through `native_glass_panel_ns_rect`
Test: `rg "native_glass_panel_ns_rect\\(panel\\.rect, container\\.rect\\)|native_glass_ns_rect_from_makepad_rect" platform/src/os/apple/macos/macos_window.rs`

### Scenario: spacing probe is documented
Given animated morphing remains a Phase H gate
When the advanced behavior gates and completion audit are inspected
Then they reference the spacing probe and still say animated morph transitions are unproven
Test: `rg "native-container-spacing|animated.*unproven|spacing.*probe" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
