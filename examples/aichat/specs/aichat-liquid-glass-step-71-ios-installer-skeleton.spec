spec: task
name: "AI Chat Liquid Glass Step 71 iOS Installer Skeleton"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, installer]
---

## Intent

Start the real UIKit native-glass backend after the iOS class and selector
preflight work. This step may instantiate UIKit Liquid Glass objects only after
runtime preflight succeeds, while keeping all API use dynamic so older SDKs keep
building.

## Decisions

- Keep the backend under the existing `SetNativeGlassBatch` platform op.
- Install UIKit glass views below the `MTKView` inside the native glass host
  view, preserving Makepad rendering and input ownership above the native
  underlay.
- Use Makepad logical units directly as UIKit point-space coordinates; UIKit and
  Makepad both use a top-left origin for this path.
- Keep v4.1 panels input passthrough by setting UIKit glass views non-interactive.
- Do not use typed iOS 26 SDK symbols.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/ios/ios.rs`
- `platform/src/os/apple/ios/ios_app.rs`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not change macOS native glass behavior.
- Do not put UIKit glass views above the `MTKView`.
- Do not enable native interactive controls.
- Do not claim Phase G complete without iOS 26 runtime visual validation.

## Acceptance Criteria

### Scenario: UIKit geometry mapping is explicit
Given a Makepad native glass container and panel
When the iOS installer converts them to UIKit frames
Then container coordinates keep the Makepad logical rect and panel coordinates
are relative to the container
Test: `rg "native_glass_ui_rect_from_makepad_rect|native_glass_panel_ui_rect|ios_native_glass_ui_rect_keeps_makepad_logical_coordinates" platform/src/os/apple/ios/ios_app.rs`

### Scenario: UIKit style values are mapped dynamically
Given native glass panel styles
When the iOS installer creates `UIGlassEffect`
Then it maps `Regular` and `Clear` to raw style values without typed iOS 26
symbols
Test: `rg "ios_native_glass_style_raw_value|initWithStyle" platform/src/os/apple/ios/ios_app.rs`

### Scenario: UIKit views are inserted below Metal
Given iOS native glass preflight passes
When `SetNativeGlassBatch` is processed
Then `IosApp` installs a native glass container into the host view below
`mtk_view`
Test: `rg "install_native_glass_batch|insertSubview|belowSubview|native_glass_host_view" platform/src/os/apple/ios/ios.rs platform/src/os/apple/ios/ios_app.rs`

### Scenario: fallback remains honest
Given iOS native glass classes or selectors are unavailable
When `SetNativeGlassBatch` is processed
Then it still reports `PreflightFailed` with the missing class or selector
Test: `rg "PreflightFailed|missing_class|missing_selector" platform/src/os/apple/ios/ios.rs`

### Scenario: iOS target compiles
Given the installer skeleton uses only dynamic Objective-C runtime calls
When iOS platform is checked
Then it compiles for `aarch64-apple-ios`
Test: `cargo check -p makepad-platform --target aarch64-apple-ios --release`
