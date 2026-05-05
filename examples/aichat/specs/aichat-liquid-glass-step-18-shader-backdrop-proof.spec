spec: task
name: "AI Chat Liquid Glass Step 18 Shader Backdrop Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, glass-panel, studio-validation]
---

## Intent

Implement the first `ShaderBackdropInterior` slice after the v4.2 route
decision. This step proves that `GlassPanel` can render an interior backdrop
signal inside its rounded shape without relying on Apple native overlays or
platform APIs.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-proof` for this diagnostic target.
- Keep normal `shader`, `macos-native`, and `macos-native-clear` behavior
  unchanged.
- Step 18 uses a procedural raw backdrop signal inside `GlassPanel`; offscreen
  texture capture, blur passes, per-panel UV mapping, and refraction are Step
  19+ work.
- The proof must be fully Makepad-rendered and must not touch `platform/src/`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-18-shader-backdrop-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not change `platform/src/**`.
- Do not rename or remove existing aichat native runnables.
- Do not implement blur/refraction in this step.
- Do not make `shader-backdrop-proof` the default backend.

### Out of Scope

- Offscreen backdrop texture pass.
- Blur passes.
- Desktop/window capture.
- Native AppKit/UIKit integration.

## Acceptance Criteria

Scenario: proof backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables backdrop proof config

Scenario: Studio exposes proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-proof|shader-backdrop-proof" makepad.splash`
Given Studio lists aichat runnables
When the shader backdrop proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-proof`

Scenario: GlassPanel has opt-in backdrop sampling
Test: `rg "backdrop_sample_strength|backdrop_mix|backdrop_signal_rgb" widgets/src/glass_panel.rs`
Given existing `GlassPanel` call sites do not opt in
When the shader is inspected
Then backdrop sampling defaults to disabled and only contributes when strength is positive

Scenario: aichat wires proof parameters only for backdrop appearance
Test: `rg "backdrop_sample_strength|backdrop_mix|ShaderBackdropConfig" examples/aichat/src/main.rs`
Given normal shader and native modes remain supported
When `ShaderBackdropConfig` is present
Then aichat enables `GlassPanel` backdrop sampling parameters

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given the proof shader and aichat wiring are added
When the affected crates are checked
Then they compile without errors
