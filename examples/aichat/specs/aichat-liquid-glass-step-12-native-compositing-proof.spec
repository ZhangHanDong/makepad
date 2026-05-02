spec: task
name: "AI Chat Liquid Glass Step 12 Native Compositing Proof"
tags: [makepad, aichat, liquid-glass, macos-native, compositing, studio-validation]
---

## Intent

Add explicit diagnostic modes for the user-reported issue where native Liquid
Glass is visible mostly at panel edges. The diagnostics must separate three
causes: Makepad overlay opacity, native glass interior sampling, and the v4.1
Metal-over-native compositing model.

## Decisions

- Keep the normal `macos-native` and `macos-native-clear` runnables unchanged.
- Add a Studio runnable that starts `macos-native-clear` with a striped native
  proof substrate and a transparent Makepad glass overlay.
- In transparent-overlay proof mode, Makepad glass panels still export native
  descriptors but draw no tint, border, highlight, noise, or halo.
- In striped proof substrate mode, AppKit draws high-contrast native views below
  the native glass and below the Makepad Metal layer.
- Treat this as evidence gathering, not a final full-Liquid-Glass fix.

## Constraints

- Must not launch UI validation through raw `cargo run`.
- Must not change default user-facing shader or native modes.
- Must not mix ShaderBackdrop with AppleNative.
- Must not make native panels interactive.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-12-native-compositing-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_window.rs`

### Forbidden

- `widgets/**`
- iOS backend files
- ShaderBackdrop implementation

### Out of Scope

- Full compositing redesign.
- Native-hosted Makepad subpasses.
- Offscreen blur/refraction passes.

## Acceptance Criteria

Scenario: transparent overlay proof disables Makepad decoration
Test: `cargo test -p makepad-example-aichat aichat_native_compositing_proof -- --nocapture`
Given `AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay`
When aichat computes native glass overlay values
Then tint, border, highlight, noise, and halo are all zero

Scenario: normal native overlay remains unchanged by default
Test: `cargo test -p makepad-example-aichat aichat_native_overlay -- --nocapture`
Given no compositing proof env is set
When aichat computes native overlay values
Then the existing native overlay tuning remains active

Scenario: Studio exposes a striped compositing proof runnable
Test: `rg "makepad-example-aichat-macos-native-clear-proof-stripes|AICHAT_NATIVE_COMPOSITING_PROOF|AICHAT_NATIVE_SUBSTRATE_PROOF" makepad.splash`
Given Studio lists runnable items
When the compositing proof target is launched
Then it runs aichat in clear native mode with transparent overlay and striped proof substrate
And it remains on the AppleNative backend instead of enabling ShaderBackdrop

Scenario: macOS proof substrate supports stripes
Test: `rg "install_proof_substrate|proof-substrate=stripes|AICHAT_NATIVE_SUBSTRATE_PROOF" platform/src/os/apple/macos`
Given `AICHAT_NATIVE_SUBSTRATE_PROOF=stripes`
When a macOS window is created
Then the platform installs high-contrast proof stripes behind the native glass

Scenario: proof docs preserve the diagnostic-only decision
Test: `rg "evidence gathering|not a final full-Liquid-Glass fix|compositing proof" examples/aichat/specs/aichat-liquid-glass-step-12-native-compositing-proof.spec`
Given the striped proof runnable exists
When the Step 12 task is reviewed
Then the spec states this is evidence gathering rather than the final full Liquid Glass fix

Scenario: affected crates still compile
Test: `cargo check -p makepad-platform && cargo check -p makepad-example-aichat`
Given compositing proof modes are implemented
When the affected crates are checked
Then they compile without errors
