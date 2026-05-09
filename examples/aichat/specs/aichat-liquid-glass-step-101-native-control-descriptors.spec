spec: task
name: "AI Chat Liquid Glass Step 101 Native Control Descriptors"
tags: [makepad, platform, liquid-glass, native-controls, descriptors]
---

## Intent

Add the shared descriptor model for future native interactive Liquid Glass
controls. This is the first code slice after the Step 100 ownership decision:
it defines native controls as a separate platform model from native glass
panels, without creating AppKit/UIKit controls yet.

## Decisions

- Native control descriptors are separate from `NativeGlassPanelDescriptor`.
- `NativeGlassControlBatch` is window-scoped and contains explicit native
  control rects.
- v4.10 validation allows at most 24 visible native controls per window.
- Visible native controls must have non-empty rects.
- Native control descriptors carry button role, style, tint, z-order, enabled,
  and visible state.
- No `CxOsOp` or platform installer is added in this step.

## Boundaries

### Allowed Changes

- `platform/src/event/window.rs`
- `platform/src/lib.rs`
- `examples/aichat/specs/aichat-liquid-glass-step-101-native-control-descriptors.spec`
- `examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Forbidden

- Do not add native AppKit/UIKit control creation.
- Do not add a platform op that installs native controls.
- Do not change native glass panel validation.
- Do not make `NativeGlassHitTest::Interactive` valid for panels.
- Do not change aichat runtime behavior.

## Out of Scope

- Collecting descriptors from `GlassButton`.
- Native action bridge implementation.
- Native accessibility implementation.
- UIKit runtime validation.

## Acceptance Criteria

Scenario: shared native control descriptor types exist
Test: `rg "NativeGlassControlBatch|NativeGlassControlDescriptor|NativeGlassControlKind|NativeGlassButtonRole" platform/src/event/window.rs platform/src/lib.rs`
Given native controls are separate from native panels
When platform event types are inspected
Then native control descriptors and batch types exist and are exported

Scenario: v4.10 native control validation exists
Test: `rg "NATIVE_GLASS_MAX_CONTROLS_PER_WINDOW_V4_10|validate_v4_10|TooManyVisibleControls|EmptyVisibleControlRect" platform/src/event/window.rs`
Given native control descriptors can be submitted in a later step
When validation is inspected
Then it caps visible controls and rejects empty visible rects

Scenario: descriptor tests cover validation and equivalence
Test: `cargo test -p makepad-platform native_glass_control -- --nocapture`
Given native control descriptors are added
When platform tests run
Then max-count validation, empty-rect validation, and update equivalence are covered

Scenario: no native control installer is added
Test: `! rg "SetNativeGlassControl|NativeGlassControlBatch" platform/src/cx_api.rs platform/src/os/apple platform/src/os/windows platform/src/os/linux`
Given this step only adds shared descriptors
When platform operations and OS backends are inspected
Then no native control install path exists yet

Scenario: completion audit distinguishes descriptors from implementation
Test: `rg "Step 101 adds shared native control descriptors|native interactive controls are not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
Given shared descriptors now exist
When the completion audit is inspected
Then it still does not claim native interactive controls are implemented
