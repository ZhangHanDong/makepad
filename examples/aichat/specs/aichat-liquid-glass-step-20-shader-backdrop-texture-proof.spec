spec: task
name: "AI Chat Liquid Glass Step 20 Shader Backdrop Texture Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, texture, studio-validation]
---

## Intent

Move the ShaderBackdrop proof from purely procedural panel color math to a real
texture sampler path. This step proves that `GlassPanel` can sample a bound
backdrop texture before the later offscreen scene pass and separable blur pass
are introduced.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-texture-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-texture-proof` for this diagnostic
  target.
- Populate the first texture proof with an app-owned CPU `Texture`; it is a
  controlled test signal, not the final offscreen backdrop scene.
- Bind the texture to the same large `GlassPanel` surfaces already used by the
  raw and blurred proof targets.
- Keep Step 18 raw and Step 19 blur proof runnables available for A/B
  comparison.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-20-shader-backdrop-texture-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not implement native platform sampling.
- Do not implement the final offscreen backdrop scene pass in this step.
- Do not implement separable Gaussian render passes in this step.
- Do not remove the Step 18 or Step 19 proof targets.
- Do not make texture proof the default backend.

### Out of Scope

- Offscreen `DrawPass` scene capture.
- Horizontal/vertical blur render passes.
- Per-panel screen-space UV mapping.
- Refraction/lensing.

## Acceptance Criteria

Scenario: texture proof backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_texture_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-texture-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables texture backdrop proof config

Scenario: Studio exposes texture proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-texture-proof|shader-backdrop-texture-proof" makepad.splash`
Given Studio lists aichat runnables
When the texture proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-texture-proof`

Scenario: GlassPanel has opt-in texture sampling
Test: `rg "backdrop_texture|backdrop_texture_strength|backdrop_texture_rgb" widgets/src/glass_panel.rs`
Given existing `GlassPanel` call sites do not opt in
When the shader is inspected
Then texture sampling defaults to disabled and only contributes when texture strength is positive

Scenario: aichat creates and binds the proof texture
Test: `rg "TextureSignal|ensure_shader_backdrop_texture|bind_shader_backdrop_texture" examples/aichat/src/main.rs`
Given the texture proof appearance is active
When the UI glass appearance is applied
Then aichat creates a deterministic texture signal and binds it to the proof panels

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given the texture proof shader and aichat wiring are added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the texture proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
