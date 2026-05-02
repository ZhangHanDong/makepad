# AIChat Liquid Glass v4 Phase D Plan

## Goal

Add Makepad widget-level native glass descriptor export while preserving existing
`GlassPanel` shader fallback behavior by default.

## Scope

- Add `GlassContainer` as the opt-in descriptor collection scope.
- Evolve `GlassPanel` into a semantic glass widget with native descriptor fields,
  defaulting to `native: false`.
- Collect native descriptors after layout and submit a `NativeGlassBatch` through
  the platform op added in Phase C.
- Keep existing `GlassPanel` call sites behavior-compatible unless `native: true`
  is explicitly set.

## Out Of Scope

- aichat shell/sidebar/main/composer migration.
- iOS/UIKit backend.
- Native button/toolbar controls.
- ShaderBackdrop blur/refraction.

## Acceptance

- `cargo test -p makepad-widgets native_glass -- --nocapture` passes.
- `cargo check -p makepad-widgets` passes.
- `cargo check -p makepad-example-aichat` passes.
- Existing `GlassPanel` users remain shader-only unless `native: true`.
