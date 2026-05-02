# AIChat Liquid Glass v4 Phase C Plan

## Goal

Implement the first macOS AppleNative container backend that consumes the shared
`NativeGlassBatch` descriptors from Phase B.

## Scope

- Add an app-to-platform operation for applying a native glass batch to a window.
- Implement the macOS backend using runtime Objective-C lookup for
  `NSGlassEffectContainerView` and `NSGlassEffectView`.
- Preserve the v4.1 compositing model: all native glass views sit below the
  Makepad Metal view and remain passthrough.
- Apply static container spacing, per-panel style, tint, shape radius, frame, and
  native-panel z-order.
- Return/log `NativeGlassBatchResult` data and keep the v1 compatibility native
  substrate event/log for existing aichat state handling.

## Out Of Scope

- Widget-tree traversal and automatic descriptor export.
- iOS/UIKit backend.
- Native button/toolbar controls.
- ShaderBackdrop blur/refraction.
- Runtime backend switching.

## Implementation Steps

1. Add failing tests for the public `WindowHandle` batch submission path and macOS
   logical-to-AppKit frame conversion.
2. Add `CxOsOp::SetNativeGlassBatch` and a `WindowHandle::set_native_glass_batch`
   helper.
3. Add macOS native container state to `MacosWindow`.
4. Add `MacosWindow::install_native_glass_batch` using runtime class/selector
   checks.
5. Wire the new platform op in macOS, with no-op handling on non-macOS backends.
6. Run targeted tests and `cargo check -p makepad-platform`.

## Acceptance

- `cargo test -p makepad-platform native_glass -- --nocapture` passes.
- `cargo check -p makepad-platform` passes.
- Unsupported runtimes return/log structured class-missing/preflight results
  without crashing.
- On supported macOS, a valid one-container batch installs multiple native panels
  below the Metal view.
