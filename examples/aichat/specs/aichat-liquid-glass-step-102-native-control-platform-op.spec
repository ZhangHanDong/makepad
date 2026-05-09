spec: task
name: "AI Chat Liquid Glass Step 102 Native Control Platform Op"
tags: [makepad, platform, liquid-glass, native-controls, platform-op]
---

## Intent

Connect the Step 101 native-control descriptor batch to the Makepad platform op
queue. This creates the shared transport needed by later `GlassButton`
collection and AppKit/UIKit installers, while keeping all OS backends as
no-op consumers for now.

## Decisions

- Add `CxOsOp::SetNativeGlassControlBatch(NativeGlassControlBatch)`.
- Add `WindowHandle::set_native_glass_control_batch`.
- Validate only window-id routing in the window API; descriptor validation
  remains on `NativeGlassControlBatch::validate_v4_10`.
- macOS, iOS, web, Windows, Linux X11, and Linux Wayland explicitly ignore the
  new op until a native-control installer is implemented.
- Do not collect `GlassButton` descriptors or create AppKit/UIKit controls in
  this step.

## Boundaries

### Allowed Changes

- `platform/src/cx_api.rs`
- `platform/src/window.rs`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/ios/ios.rs`
- `platform/src/os/web/web.rs`
- `platform/src/os/windows/windows.rs`
- `platform/src/os/linux/x11/linux_x11.rs`
- `platform/src/os/linux/wayland/linux_wayland.rs`
- `examples/aichat/specs/aichat-liquid-glass-step-102-native-control-platform-op.spec`
- `examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Forbidden

- Do not implement native AppKit/UIKit controls.
- Do not install native controls in any OS backend.
- Do not change native glass panel behavior.
- Do not change aichat runtime behavior.

## Out of Scope

- `GlassButton` descriptor collection.
- AppKit/UIKit native button creation.
- Native action bridge.
- Accessibility bridge.

## Acceptance Criteria

Scenario: `CxOsOp::SetNativeGlassControlBatch` and `WindowHandle::set_native_glass_control_batch` exist
Test: `rg "SetNativeGlassControlBatch|set_native_glass_control_batch|NativeGlassControlBatch" platform/src/cx_api.rs platform/src/window.rs`
Given native control descriptors exist
When platform queue APIs are inspected
Then a dedicated native-control batch op and window API are available

Scenario: window API queues native control batch for created windows
Test: `cargo test -p makepad-platform set_native_glass_control_batch -- --nocapture`
Given a created window receives a native control batch with the matching window id
When the window API is called
Then it queues `CxOsOp::SetNativeGlassControlBatch`

Scenario: current OS backends ignore the op
Test: `rg "SetNativeGlassControlBatch\\(_\\) => \\{\\}|SetNativeGlassBatch\\(_\\) \\| CxOsOp::SetNativeGlassControlBatch\\(_\\)" platform/src/os/apple/macos/macos.rs platform/src/os/apple/ios/ios.rs platform/src/os/web/web.rs platform/src/os/windows/windows.rs platform/src/os/linux/x11/linux_x11.rs platform/src/os/linux/wayland/linux_wayland.rs`
Given native control installers are not implemented yet
When OS backend platform-op matches are inspected
Then each current backend has an explicit no-op arm for native control batches

Scenario: window API rejects native control batches for the wrong window
Test: `cargo test -p makepad-platform set_native_glass_control_batch_rejects_window_id_mismatch -- --nocapture`
Given a native control batch targets a different window id
When the window API is called
Then it does not queue `CxOsOp::SetNativeGlassControlBatch`

Scenario: completion audit distinguishes transport from implementation
Test: `rg "Step 102 adds the platform op transport|native interactive controls are not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
Given the platform op transport exists
When the completion audit is inspected
Then it still does not claim native interactive controls are implemented
