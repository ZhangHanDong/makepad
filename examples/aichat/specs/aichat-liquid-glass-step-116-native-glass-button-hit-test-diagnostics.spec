spec: task
name: "AI Chat Liquid Glass Step 116 Native Glass Button Hit-Test Diagnostics"
tags: [makepad, macos, liquid-glass, native-controls, hit-test, diagnostics]
---

## Intent

Replace the macOS native-control mirror from a raw `NSButton` allocation with a
thin `NativeGlassButton` subclass that exposes AppKit hit-test evidence and
accepts first mouse for inactive-window validation.

## Decisions

- `NativeGlassButton` subclasses `NSButton`.
- `acceptsFirstMouse:` returns `YES` and logs a diagnostic.
- `hitTest:` logs the point and the AppKit class returned by super.
- `mouseDown:` logs the window point before forwarding to super.

## Boundaries

- Do not change native control descriptors.
- Do not change Makepad button semantics.
- Do not move visual glass panels above Metal.
- Do not claim native control validation until runtime logs prove delivery.

## Out of Scope

- macOS 26 glass-specific button styling.
- UIKit control subclasses.
- Accessibility ownership.
- Synthetic event forwarding from Makepad to AppKit.

## Acceptance Criteria

### Scenario: native button subclass exists

Test: `rg "define_native_glass_button_class|NativeGlassButton" platform/src/os/apple/macos/macos_delegates.rs platform/src/os/apple/macos/macos_app.rs`

Given macOS native controls are installed
When the button class is requested
Then the backend uses a custom `NativeGlassButton` subclass.

### Scenario: first mouse and hit test diagnostics exist

Test: `rg "accepts-first-mouse|button-hit-test|button-mouse-down" platform/src/os/apple/macos/macos_delegates.rs`

Given a system click reaches the native button
When AppKit performs hit testing and mouse delivery
Then the subclass logs first-mouse, hit-test, and mouse-down diagnostics.

### Scenario: installer uses subclass

Test: `rg "native_glass_button" platform/src/os/apple/macos/macos_window.rs platform/src/os/apple/macos/macos_app.rs`

Given the macOS native control installer
When allocating button views
Then it allocates the registered native-glass button subclass.
