spec: task
name: "AI Chat Liquid Glass Step 03 macOS Installer"
tags: [makepad, aichat, liquid-glass, macos, native-installer, phase-c]
---

## Intent

Connect the shared native glass batch descriptor to the platform operation path
and implement the macOS installer that creates `NSGlassEffectContainerView` and
child `NSGlassEffectView` instances below the Metal layer. Non-macOS backends
must accept the new operation as a no-op so the shared API remains portable.

## Decisions

- Add `CxOsOp::SetNativeGlassBatch(NativeGlassBatch)` as the platform queue
  operation.
- Add `WindowHandle::set_native_glass_batch` as the public queueing helper.
- On macOS, install all native glass views below `metal_view` so Makepad content
  remains above the native glass panels.
- Convert Makepad top-left logical rects into AppKit bottom-left coordinates
  relative to the single v4.1 container.
- Use sRGB when converting `Vec4f` tint into `NSColor`.
- Clear the previous native batch before installing a replacement batch.
- Keep Linux, Windows, and Web behavior as no-op handling for this operation.

## Constraints

- Must not use AppKit/UIKit types in shared platform descriptor structs.
- Must validate `NativeGlassBatch::validate_v4_1` before creating native views.
- Must keep native glass panels passthrough in v4.1; `Interactive` descriptors
  are rejected before installation.
- Must return/log per-container and per-panel installation results.
- Must keep the v1 `WindowNativeSubstrateResolvedEvent` compatibility event for
  existing aichat state handling.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-03-macos-installer.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-phase-c.plan.md`
- `platform/src/cx_api.rs`
- `platform/src/window.rs`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_window.rs`
- `platform/src/os/linux/wayland/linux_wayland.rs`
- `platform/src/os/linux/x11/linux_x11.rs`
- `platform/src/os/web/web.rs`
- `platform/src/os/windows/windows.rs`

### Forbidden

- `widgets/**`
- `examples/aichat/src/**`
- `platform/src/event/window.rs`
- `platform/src/lib.rs`

### Out of Scope

- Widget traversal and descriptor export.
- aichat visual tuning.
- iOS native installer implementation.
- ShaderBackdrop implementation.

## Acceptance Criteria

Scenario: WindowHandle queues native glass batches
Test:
  Package: makepad-platform
  Filter: set_native_glass_batch_queues_platform_op_for_created_window
Given a created `WindowHandle`
When `set_native_glass_batch` is called with a `NativeGlassBatch`
Then a `CxOsOp::SetNativeGlassBatch` operation is queued for the platform layer

Scenario: macOS installer uses native container and panel views
Test: `rg "NSGlassEffectContainerView|NSGlassEffectView|setSpacing:|setCornerRadius:|setStyle:" platform/src/os/apple/macos/macos_window.rs`
Given a valid native glass batch on macOS
When the installer runs
Then it resolves the native glass classes and applies spacing, corner radius, and style selectors

Scenario: non-macOS backends handle the operation as no-op
Test: `rg "SetNativeGlassBatch\\(_\\) => \\{\\}" platform/src/os/linux/wayland/linux_wayland.rs platform/src/os/linux/x11/linux_x11.rs platform/src/os/web/web.rs platform/src/os/windows/windows.rs`
Given the shared operation exists on all platforms
When non-macOS platform op handlers are searched
Then each backend handles `SetNativeGlassBatch` without attempting native installation

Scenario: platform crate compiles with installer wiring
Test: `cargo check -p makepad-platform`
Given the new platform op and backend handlers are present
When the platform crate is checked
Then all platform op match arms compile

Scenario: Step 03 commit is limited to installer plumbing
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-03-macos-installer.spec|examples/aichat/specs/aichat-liquid-glass-v4-phase-c.plan.md|platform/src/cx_api.rs|platform/src/window.rs|platform/src/os/apple/macos/macos.rs|platform/src/os/apple/macos/macos_window.rs|platform/src/os/linux/wayland/linux_wayland.rs|platform/src/os/linux/x11/linux_x11.rs|platform/src/os/web/web.rs|platform/src/os/windows/windows.rs)$" && exit 1 || exit 0`
Given only Step 03 files are staged
When the staged file list is checked
Then widget and aichat application files are absent
