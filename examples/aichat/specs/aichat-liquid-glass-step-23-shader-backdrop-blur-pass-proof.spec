spec: task
name: "AI Chat Liquid Glass Step 23 Shader Backdrop Blur Pass Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, blur, draw-pass, studio-validation]
---

## Intent

Add a two-pass separable blur chain for the offscreen backdrop scene. This
proves that aichat can produce a blurred render texture from the Step 22 raw
scene before the blurred result is used as the production panel sampling source.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-blur-pass-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-blur-pass-proof` for this
  diagnostic target.
- Use two app-owned `DrawPass` instances: one horizontal blur pass and one
  vertical blur pass.
- Use the Step 22 offscreen scene texture as the blur source.
- Keep the blur shader small and deterministic; optimize only after visual
  proof.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-23-shader-backdrop-blur-pass-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not implement edge refraction in this step.
- Do not make blurred backdrop sampling the default backend.
- Do not remove Step 22 raw scene proof.
- Do not add external shader or image dependencies.

## Out of Scope

- Multi-resolution blur optimization.
- Kawase/downsample blur tuning.
- Refraction/lensing.
- Apple native backend changes.

## Acceptance Criteria

Scenario: blur pass backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_blur_pass_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-blur-pass-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables blur pass proof config

Scenario: Studio exposes blur pass proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-blur-pass-proof|shader-backdrop-blur-pass-proof" makepad.splash examples/aichat/src/main.rs`
Given Studio lists aichat runnables
When the blur pass proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-blur-pass-proof`

Scenario: aichat owns horizontal and vertical blur passes
Test: `rg "shader_backdrop_blur_h_pass|shader_backdrop_blur_v_pass|DrawAichatBackdropBlur" examples/aichat/src/main.rs`
Given the blur pass proof is active
When the code is inspected
Then aichat renders horizontal and vertical blur passes from the raw scene texture

Scenario: blur pass proof does not enable refraction
Test: `rg "ShaderBackdropProof::BlurPass|backdrop_refraction_strength" examples/aichat/src/main.rs widgets/src/glass_panel.rs`
Given the blur pass proof is active
When the proof config is inspected
Then blur is enabled and refraction remains disabled

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given blur pass shaders and wiring are added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
