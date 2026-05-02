spec: task
name: "AI Chat Liquid Glass Step 09 Native Batch Throttle"
tags: [makepad, widgets, liquid-glass, studio-validation, regression]
---

## Intent

Fix the Studio validation regression where `GlassContainer` submits the same
`SetNativeGlassBatch` every draw frame, causing macOS to reinstall native glass
views and flood the Studio bridge logs. The widget should submit native batch
updates only when the batch descriptor changes.

## Decisions

- Cache the last submitted `NativeGlassBatch` inside `GlassContainer`.
- Submit `CxOsOp::SetNativeGlassBatch` only when the new batch differs from the
  cached batch.
- Reset the cached batch when the container is no longer native.
- Keep resize/layout updates working because rect changes produce a different
  batch.

## Constraints

- Must not change macOS installer behavior for this fix.
- Must not remove native batch logging from the platform backend.
- Must keep widget and aichat crates compiling.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-09-native-batch-throttle.spec`
- `widgets/src/glass_panel.rs`

### Forbidden

- `platform/**`
- `examples/aichat/src/**`

### Out of Scope

- Visual tuning.
- Screenshot validation.
- Native clear style validation.

## Acceptance Criteria

Scenario: GlassContainer caches the last native batch
Test: `rg "last_native_batch|SetNativeGlassBatch\\(batch.clone\\(\\)\\)|self.last_native_batch = Some\\(batch\\)|self.last_native_batch = None" widgets/src/glass_panel.rs`
Given a native `GlassContainer` draws multiple frames without layout changes
When the second frame produces an equal `NativeGlassBatch`
Then it does not submit another platform op

Scenario: widget native glass tests still pass
Test:
  Package: makepad-widgets
  Filter: native_glass
Given native descriptor collection changed
When widget tests run
Then existing collector and shape tests pass

Scenario: aichat still compiles with throttled widget behavior
Test: `cargo check -p makepad-example-aichat`
Given aichat uses `GlassContainer` and native panels
When the example is checked
Then the throttle does not break script or Rust integration
