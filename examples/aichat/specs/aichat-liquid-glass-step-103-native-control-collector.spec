spec: task
name: "AI Chat Liquid Glass Step 103 Native Control Collector Plumbing"
tags: [makepad, widgets, liquid-glass, native-controls, collector]
---

## Intent

Extend the existing `GlassContainer` native collector plumbing so it can carry
future native control descriptors alongside native glass panel descriptors.
This keeps `GlassButton` collection as a later widget step, while making the
container and platform-op route ready for that data.

## Decisions

- Reuse the existing `NativeGlassCollector` stack owned by `GlassContainer`.
- Store native controls separately from native panels in `NativeGlassCollection`.
- `GlassContainer` emits `CxOsOp::SetNativeGlassControlBatch` only when the
  collected control batch is non-empty or when a previous control batch must be
  cleared.
- Do not change the current `GlassButton` alias or runtime behavior in this
  step.
- Do not create native AppKit/UIKit controls in this step.

## Boundaries

### Allowed Changes

- `widgets/src/glass_panel.rs`
- `examples/aichat/specs/aichat-liquid-glass-step-103-native-control-collector.spec`
- `examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Forbidden

- Do not refactor `GlassButton` from a styled `ButtonFlat` alias into a custom
  Rust widget in this step.
- Do not create or install native AppKit/UIKit controls.
- Do not change native glass panel validation or behavior.
- Do not change aichat runtime behavior.

## Out of Scope

- `GlassButton` descriptor export.
- Native action bridge.
- Native accessibility bridge.
- macOS/iOS native control installers.

## Acceptance Criteria

Scenario: collector stores native controls separately from panels
Test: `rg "controls: Vec<NativeGlassControlDescriptor>|push_control|NativeGlassControlBatch" widgets/src/glass_panel.rs`
Given `GlassContainer` already collects native panels
When the collector model is inspected
Then it also has a separate native-control descriptor list

Scenario: GlassContainer emits native control batches through the platform op
Test: `rg "SetNativeGlassControlBatch|last_native_control_batch|control_batch.controls.is_empty" widgets/src/glass_panel.rs`
Given future widgets push native control descriptors
When `GlassContainer` finishes a native collection
Then it can queue `CxOsOp::SetNativeGlassControlBatch`

Scenario: current GlassButton alias is unchanged
Test: `rg "mod.widgets.GlassButton = mod.widgets.ButtonFlat" widgets/src/glass_panel.rs`
Given native control descriptors are not exported by buttons yet
When the widget definitions are inspected
Then `GlassButton` remains the existing `ButtonFlat` alias

Scenario: collector unit test covers pushed controls
Test: `cargo test -p makepad-widgets native_glass_collector_returns_container_with_pushed_controls -- --nocapture`
Given a native control descriptor is pushed into the collector
When the collection finishes
Then the control descriptor is preserved separately from panels

Scenario: collector finish without begin is a no-op
Test: `cargo test -p makepad-widgets native_glass_collector_finish_without_begin_returns_none -- --nocapture`
Given no native collection has begun
When the collector is asked to finish
Then it returns no collection and queues no native controls

Scenario: completion audit distinguishes collector plumbing from implementation
Test: `rg "Step 103 extends GlassContainer collector plumbing|native interactive controls are not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
Given control collector plumbing exists
When the completion audit is inspected
Then it still does not claim native interactive controls are implemented
