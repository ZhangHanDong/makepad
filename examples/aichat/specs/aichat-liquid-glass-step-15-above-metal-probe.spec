spec: task
name: "AI Chat Liquid Glass Step 15 Above Metal Probe"
tags: [makepad, aichat, liquid-glass, macos-native, compositing, studio-validation]
---

## Intent

Implement the first v4.2 compositing prototype: an above-Metal sampling probe.
The probe places a native `NSGlassEffectView` above the Makepad Metal view while
aichat draws a high-contrast moving Metal pattern underneath it. This determines
whether AppKit native glass can visibly sample Makepad-rendered Metal content.

## Decisions

- Keep normal aichat and v4.1 native targets unchanged.
- Add a dedicated Studio runnable for the probe.
- Use `AICHAT_NATIVE_ABOVE_METAL_PROBE=clear` to request the diagnostic native
  glass overlay.
- Use `AICHAT_METAL_PROBE_PATTERN=moving-pattern` to show a Metal-rendered
  high-contrast pattern behind the probe.
- Document the probe as diagnostic; it may cover Makepad content and is not a
  user-facing UI mode.

## Constraints

- Must not use raw `cargo run` for UI validation.
- Must not rename existing native targets.
- Must not claim full native Liquid Glass from this probe alone.
- Must keep the probe separate from ShaderBackdrop.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-15-above-metal-probe.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_window.rs`

### Forbidden

- `widgets/**`
- iOS backend files
- ShaderBackdrop implementation

### Out of Scope

- Two-layer interleave probe.
- Input forwarding policy.
- aichat production UI integration.

## Acceptance Criteria

Scenario: aichat can draw a Metal probe pattern
Test: `cargo test -p makepad-example-aichat aichat_above_metal_probe -- --nocapture`
Given `AICHAT_METAL_PROBE_PATTERN=moving-pattern`
When aichat starts
Then the Metal-rendered probe pattern view can be enabled without changing normal mode

Scenario: macOS can install above-Metal probe geometry
Test: `cargo test -p makepad-platform above_metal_probe -- --nocapture`
Given a macOS window bounds rect
When the above-Metal probe frame is calculated
Then the diagnostic glass frame is inset inside the Makepad Metal view bounds

Scenario: Studio exposes the above-Metal probe runnable
Test: `rg "makepad-example-aichat-macos-native-above-metal-probe|AICHAT_NATIVE_ABOVE_METAL_PROBE|AICHAT_METAL_PROBE_PATTERN" makepad.splash`
Given Studio lists aichat runnables
When the above-Metal probe target is launched
Then it requests the native glass overlay and moving Metal probe pattern
And UI validation remains a Studio RunItem flow rather than an ad hoc raw cargo command

Scenario: macOS above-Metal probe logs install state
Test: `rg "requested_above_metal_glass_probe_style_from_env|install_above_metal_glass_probe|above-metal-probe state=installed" platform/src/os/apple/macos`
Given `AICHAT_NATIVE_ABOVE_METAL_PROBE=clear`
When a macOS window is created
Then the platform installs a diagnostic `NSGlassEffectView` above the Metal view

Scenario: affected crates still compile
Test: `cargo check -p makepad-platform && cargo check -p makepad-example-aichat`
Given the probe code is added
When the affected crates are checked
Then they compile without errors
