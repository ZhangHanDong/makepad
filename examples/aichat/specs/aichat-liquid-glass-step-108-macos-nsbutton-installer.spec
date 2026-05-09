spec: task
name: "AI Chat Liquid Glass Step 108 macOS NSButton Installer Skeleton"
tags: [makepad, macos, liquid-glass, native-controls, appkit]
---

## Intent

Install a minimal AppKit `NSButton` mirror for visible native control
descriptors on macOS, including target/action wiring back into
`NativeGlassControlActivatedEvent`. This is the first native-control installer
slice and intentionally does not claim macOS 26 glass-specific button styling.

## Decisions

- macOS creates `NSButton` views for visible `NativeGlassControlDescriptor`
  entries.
- Buttons are placed above the Metal view so AppKit can own hit testing inside
  explicit native control rects.
- A `NativeGlassControlTarget` Objective-C class posts
  `NativeGlassControlActivatedEvent`.
- `NativeGlassControlDescriptor.label` becomes the `NSButton` title.
- This step logs `installed-appkit-buttons` only for AppKit button installation,
  not for glass-specific native button appearance.

## Boundaries

- Do not enable aichat buttons as native controls in this step.
- Do not claim `NSButton` glass bezel/configuration support.
- Do not change ordinary glass panels; they remain passthrough under Metal.
- Do not implement UIKit control installation in this step.

## Out of Scope

- macOS 26 glass-specific button styling selectors.
- Native hover/down/focus state synchronization.
- Icon/image transport for native buttons.
- Accessibility ownership.

## Acceptance Criteria

### Scenario: macOS registers native control target class

Test: `rg "NativeGlassControlTarget|nativeGlassControlAction|NativeGlassControlActivatedEvent" platform/src/os/apple/macos/macos_delegates.rs platform/src/os/apple/macos/macos_app.rs`

Given a native AppKit control action
When the target receives the action callback
Then it posts `NativeGlassControlActivatedEvent` with window and control ids.

### Scenario: macOS installs NSButton mirrors

Test: `rg "NSButton|install_native_glass_button_control|installed-appkit-buttons" platform/src/os/apple/macos/macos_window.rs`

Given a valid native control batch with visible button descriptors
When macOS processes it
Then it creates `NSButton` views and logs `installed-appkit-buttons` on success.

### Scenario: macOS keeps control hit testing explicit

Test: `rg "positioned: 1i64|relativeTo: self.view" platform/src/os/apple/macos/macos_window.rs`

Given native control views are installed
When they are added to the AppKit hierarchy
Then they are placed above the Metal view only for explicit control rects.

### Scenario: aichat remains opt-in

Test: `rg "native_control:" examples/aichat/src widgets/src -g '*.rs'`

Given this is an installer skeleton
When reviewing app/widget defaults
Then aichat has not enabled native controls by default.
