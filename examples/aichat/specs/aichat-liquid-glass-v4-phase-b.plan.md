# AIChat Liquid Glass v4 Phase B Plan

## Goal

Implement the platform-independent native Liquid Glass descriptor API described by
`aichat-liquid-glass-v4-apple-native-full.spec.md`, without adding AppKit/UIKit
types or widget collection logic.

## Scope

- Add shared descriptor, validation, backend state, and install result types to
  `makepad-platform`.
- Keep the API backend-neutral so macOS/iOS platform code can consume it later.
- Enforce v4.1 constraints at the descriptor batch boundary:
  - `containers.len()` must be `0` or `1`.
  - At most 12 visible native panels per window.
  - `NativeGlassHitTest::Interactive` is rejected in v4.1.
- Preserve existing `WindowNativeSubstrate*` v1 event types for compatibility.

## Out Of Scope

- Widget tree traversal and automatic descriptor export.
- AppKit/UIKit native view installation.
- Shader backdrop/refraction backend.
- GlassButton/GlassToolbar native control behavior.

## Implementation Steps

1. Add failing unit tests for v4.1 batch validation and install result counting.
2. Add descriptor enums/structs and validation helpers in `platform/src/event/window.rs`.
3. Re-export the new public types from `platform/src/lib.rs`.
4. Run targeted `makepad-platform` tests and a package check.

## Acceptance

- `cargo test -p makepad-platform native_glass -- --nocapture` passes.
- `cargo check -p makepad-platform` passes.
- No AppKit/UIKit types are introduced in the shared API.
- Existing v1 substrate event types remain available.

## Implementation Status

Landed evidence:

- Step 02 task spec: `aichat-liquid-glass-step-02-platform-descriptors.spec`
- Commit: `d37d5a5a Add native glass descriptor batch API`
- Implemented files: `platform/src/event/window.rs`, `platform/src/lib.rs`
- Verification used during landing:
  - `cargo test -p makepad-platform native_glass_batch -- --nocapture`
  - `cargo test -p makepad-platform native_glass_style_maps_to_macos_raw_values -- --nocapture`
  - staged boundary check for Phase B files
