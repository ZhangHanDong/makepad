spec: task
name: "AI Chat Liquid Glass Step 22 Shader Backdrop Offscreen Scene Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, offscreen, draw-pass, studio-validation]
---

## Intent

Replace the Step 21 synthetic CPU texture source with a Makepad-rendered
offscreen backdrop scene texture. This proves that `GlassPanel` can sample a
GPU render target produced by a dedicated backdrop pass before blur or
refraction are added.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-scene-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-scene-proof` for this diagnostic
  target.
- Render the proof scene with an app-owned `DrawPass` and `RenderBGRAu8`
  texture.
- Use the existing Step 21 screen-space UV mapping for panel sampling.
- Keep Step 20 and Step 21 proof targets unchanged.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-22-shader-backdrop-offscreen-scene-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not implement blur passes in this step.
- Do not implement refraction or lensing in this step.
- Do not sample the real desktop or OS window background.
- Do not make the offscreen scene proof the default backend.
- Do not remove existing shader or native proof runnables.

## Out of Scope

- Separable blur pass chain.
- Blurred texture sampling.
- Edge refraction and rim distortion.
- Apple native interleave renderer changes.

## Acceptance Criteria

Scenario: offscreen scene backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_offscreen_scene_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-scene-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables offscreen scene backdrop proof config

Scenario: Studio exposes offscreen scene proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-scene-proof|shader-backdrop-scene-proof" makepad.splash examples/aichat/src/main.rs`
Given Studio lists aichat runnables
When the offscreen scene proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-scene-proof`

Scenario: aichat owns a render-to-texture backdrop pass
Test: `rg "shader_backdrop_scene_pass|RenderBGRAu8|DrawAichatBackdropScene" examples/aichat/src/main.rs`
Given the offscreen scene proof is active
When the code is inspected
Then aichat allocates a render texture and draws a backdrop scene into it

Scenario: offscreen scene texture is sampled by GlassPanel
Test: `rg "bind_shader_backdrop_render_texture|ScreenTextureSignal|OffscreenScene" examples/aichat/src/main.rs`
Given the offscreen scene texture exists
When glass appearance is applied or redrawn
Then proof panels receive that render texture with screen-space mapping enabled

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given the offscreen scene pass and shader are added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
