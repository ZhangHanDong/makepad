spec: task
name: "AI Chat Liquid Glass Step 24 Shader Backdrop Blurred Sampling Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, blurred-texture, sampling, studio-validation]
---

## Intent

Switch `GlassPanel` sampling from the raw offscreen scene texture to the
blurred texture produced by the Step 23 blur chain. This is the first
ShaderBackdropInterior proof where panels should read as blurred interior
glass rather than a diagnostic grid or raw scene sample.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-blurred-texture-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-blurred-texture-proof` for this
  diagnostic target.
- Use the vertical blur output as the texture bound to `GlassPanel`.
- Keep Step 22 raw scene and Step 23 blur pass runnables available for A/B
  comparison.
- Keep refraction disabled in this step.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-24-shader-backdrop-blurred-sampling-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not add edge refraction in this step.
- Do not make blurred sampling the default backend.
- Do not remove raw scene or blur pass proof targets.
- Do not sample real desktop pixels.

## Out of Scope

- Refraction/lensing.
- Blur performance optimization.
- Dynamic quality scaling.
- Native Apple backend changes.

## Acceptance Criteria

Scenario: blurred texture backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_blurred_texture_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-blurred-texture-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables blurred texture sampling proof config

Scenario: Studio exposes blurred texture proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-blurred-texture-proof|shader-backdrop-blurred-texture-proof" makepad.splash examples/aichat/src/main.rs`
Given Studio lists aichat runnables
When the blurred texture proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-blurred-texture-proof`

Scenario: GlassPanel receives the blurred output texture
Test: `rg "BlurredTexture|bind_shader_backdrop_render_texture|shader_backdrop_blur_v_texture" examples/aichat/src/main.rs`
Given the blurred texture proof is active
When aichat renders the backdrop pass chain
Then proof panels receive the vertical blur texture as their backdrop source

Scenario: blurred texture proof keeps screen-space UV mapping
Test: `rg "backdrop_texture_screen_space|set_backdrop_texture_mapping" examples/aichat/src/main.rs widgets/src/glass_panel.rs`
Given the blurred texture proof is active
When the bound texture is sampled by a panel
Then sampling uses the existing screen-space UV mapping

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given blurred texture sampling is wired
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
