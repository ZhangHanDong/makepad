spec: task
name: "AI Chat Liquid Glass Step 117 Metal View Hit-Test Diagnostics"
tags: [makepad, macos, liquid-glass, native-controls, hit-test, diagnostics]
---

## Intent

Add probe-gated diagnostics to the Makepad macOS Metal view so native-control
validation can distinguish clicks that reach the Metal view from clicks that
reach AppKit native controls.

## Decisions

- Diagnostics are gated by `AICHAT_NATIVE_CONTROL_PROBE`.
- The Metal view logs `metal-view-hit-test` with the point and resulting AppKit
  class.
- The Metal view logs `metal-view-mouse-down` with the window point.
- These diagnostics do not change event routing.

## Boundaries

- Do not change Makepad mouse event semantics.
- Do not make native controls default.
- Do not synthesize native control actions from Metal-view clicks.
- Do not log normal non-probe clicks.

## Out of Scope

- Fixing the native control hit-test issue.
- macOS 26 glass button styling.
- UIKit hit-test diagnostics.
- Accessibility ownership.

## Acceptance Criteria

### Scenario: metal view hit test is logged under probe

Test: `rg "metal-view-hit-test" platform/src/os/apple/macos/macos_delegates.rs`

Given `AICHAT_NATIVE_CONTROL_PROBE` is enabled
When AppKit hit-tests the Makepad Metal view
Then a probe-gated hit-test diagnostic is logged.

### Scenario: metal view mouse down is logged under probe

Test: `rg "metal-view-mouse-down" platform/src/os/apple/macos/macos_delegates.rs`

Given `AICHAT_NATIVE_CONTROL_PROBE` is enabled
When the Makepad Metal view receives a mouse down
Then a probe-gated mouse-down diagnostic is logged.

### Scenario: diagnostics are probe gated

Test: `rg "native_glass_control_probe_enabled" platform/src/os/apple/macos/macos_delegates.rs`

Given normal Makepad macOS apps
When the native control probe is not enabled
Then these Metal view diagnostics stay silent.
