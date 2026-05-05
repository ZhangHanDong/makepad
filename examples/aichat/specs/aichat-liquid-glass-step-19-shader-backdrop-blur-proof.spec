spec: task
name: "AI Chat Liquid Glass Step 19 Shader Backdrop Blur Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, blur, studio-validation]
---

## Intent

Extend the Step 18 raw shader-backdrop proof with a visibly softened backdrop
signal. This step proves that the `GlassPanel` sampling path can support blur
semantics before introducing offscreen texture passes and per-panel UV mapping.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-blur-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-blur-proof` for this diagnostic
  target.
- Implement blur as multi-tap procedural sampling inside `GlassPanel` for this
  step only.
- Keep Step 18 raw sampling runnable available for A/B comparison.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-19-shader-backdrop-blur-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not implement native platform sampling.
- Do not remove the Step 18 raw proof target.
- Do not claim this is the final offscreen texture blur pass.
- Do not make blur proof the default backend.

### Out of Scope

- Offscreen backdrop texture pass.
- Separable Gaussian render passes.
- Per-panel screen-space UV mapping.
- Refraction/lensing.

## Acceptance Criteria

Scenario: blur proof backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_blur_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-blur-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables blurred backdrop proof config

Scenario: Studio exposes blur proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-blur-proof|shader-backdrop-blur-proof" makepad.splash`
Given Studio lists aichat runnables
When the blur proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-blur-proof`

Scenario: GlassPanel has opt-in procedural blur sampling
Test: `rg "backdrop_blur_radius|backdrop_signal_blurred_rgb|backdrop_blur_mix" widgets/src/glass_panel.rs`
Given existing `GlassPanel` call sites do not opt in
When the shader is inspected
Then blur sampling defaults to disabled and only contributes when radius is positive

Scenario: aichat wires blur parameters only for blur proof appearance
Test: `rg "BlurredSignal|backdrop_blur_radius|backdrop_blur_mix" examples/aichat/src/main.rs`
Given raw proof and normal shader modes remain supported
When `ShaderBackdropProof::BlurredSignal` is active
Then aichat enables blur parameters for the proof target

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given the procedural blur proof shader and aichat wiring are added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the blur proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
