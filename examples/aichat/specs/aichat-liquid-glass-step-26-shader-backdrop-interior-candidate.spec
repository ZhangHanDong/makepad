spec: task
name: "AI Chat Liquid Glass Step 26 Shader Backdrop Interior Candidate"
tags: [makepad, aichat, liquid-glass, shader-backdrop, interior, candidate, studio-validation]
---

## Intent

Promote the Step 25 refraction proof into a named ShaderBackdropInterior
candidate backend. This provides a stable runnable and environment value for
manual tuning while keeping all proof targets and the default shader backend
unchanged.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-interior`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-interior` for the candidate target.
- Reuse the Step 22-25 pass chain: offscreen scene, horizontal blur, vertical
  blur, screen-space `GlassPanel` sampling, refraction, and rim.
- Keep `shader-backdrop-refraction-proof` available as the diagnostic proof
  target.
- Do not make `shader-backdrop-interior` the default backend.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-26-shader-backdrop-interior-candidate.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not remove Step 18-25 proof targets.
- Do not alter macOS native underlay behavior.
- Do not introduce Apple native interleave or native hit-test ownership.
- Do not sample real desktop pixels.
- Do not change the default `AICHAT_GLASS_BACKEND=shader` behavior.

## Out of Scope

- Final visual tuning thresholds.
- Runtime quality controls.
- Multi-resolution blur optimization.
- Making ShaderBackdropInterior production default.

## Acceptance Criteria

Scenario: shader backdrop interior backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_interior -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-interior`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables the ShaderBackdropInterior candidate config

Scenario: Studio exposes shader backdrop interior runnable
Test: `rg "makepad-example-aichat-shader-backdrop-interior|shader-backdrop-interior" makepad.splash examples/aichat/src/main.rs`
Given Studio lists aichat runnables
When the ShaderBackdropInterior target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-interior`

Scenario: interior candidate reuses the complete pass chain
Test: `rg "ShaderBackdropProof::Interior|shader_backdrop_proof_uses_render_pass|shader_backdrop_proof_uses_blur_pass|shader_backdrop_proof_samples_blurred_texture" examples/aichat/src/main.rs`
Given the interior candidate is active
When the proof routing helpers are inspected
Then it uses render pass, blur pass, blurred texture sampling, refraction, and rim routing

Scenario: default shader backend remains unchanged
Test: `cargo test -p makepad-example-aichat aichat_shader_backend_resolves_to_default_shader -- --nocapture`
Given no special glass backend is requested
When aichat resolves startup glass appearance
Then it keeps the shader substrate without backdrop config

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given the interior backend alias is added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the candidate is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
