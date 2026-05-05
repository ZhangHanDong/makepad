spec: task
name: "AI Chat Liquid Glass Step 21 Shader Backdrop Screen UV Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, texture, uv, studio-validation]
---

## Intent

Advance the Step 20 texture sampler proof from panel-local UVs to window
screen-space UVs. This proves that multiple `GlassPanel` surfaces can sample
different regions of the same backdrop texture before refraction or offscreen
scene capture are added.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-screen-texture-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-screen-texture-proof` for this
  diagnostic target.
- Keep `shader-backdrop-texture-proof` as the panel-local A/B target.
- Use the current window inner size as the proof backdrop coordinate space.
- Do not implement refraction in this step; screen-space texture mapping must be
  proven first.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-21-shader-backdrop-screen-uv-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not remove Step 18, Step 19, or Step 20 proof targets.
- Do not implement native platform sampling.
- Do not implement offscreen `DrawPass` scene capture.
- Do not implement separable blur passes.
- Do not implement refraction/lensing.
- Do not make screen-space texture proof the default backend.

### Out of Scope

- Offscreen backdrop scene pass.
- Horizontal/vertical blur render passes.
- Edge-weighted refraction.
- Real desktop/window-behind sampling.

## Acceptance Criteria

Scenario: screen-space texture proof backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_screen_texture_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-screen-texture-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables screen-space texture backdrop proof config

Scenario: Studio exposes screen-space texture proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-screen-texture-proof|shader-backdrop-screen-texture-proof" makepad.splash`
Given Studio lists aichat runnables
When the screen-space texture proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-screen-texture-proof`

Scenario: GlassPanel has opt-in screen-space texture UVs
Test: `rg "backdrop_texture_screen_space|backdrop_texture_size|backdrop_texture_uv" widgets/src/glass_panel.rs`
Given existing `GlassPanel` call sites do not opt in
When the shader is inspected
Then texture sampling remains panel-local by default and switches to screen-space only when enabled

Scenario: aichat wires screen-space texture parameters
Test: `rg "ScreenTextureSignal|backdrop_texture_screen_space|backdrop_texture_size" examples/aichat/src/main.rs`
Given the screen-space texture proof appearance is active
When glass appearance is applied
Then aichat enables screen-space sampling and passes the window size to proof panels

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given screen-space UV shader parameters and aichat wiring are added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
