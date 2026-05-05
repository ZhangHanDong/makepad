spec: task
name: "AI Chat Liquid Glass Step 25 Shader Backdrop Refraction Proof"
tags: [makepad, aichat, liquid-glass, shader-backdrop, refraction, rim, studio-validation]
---

## Intent

Add an opt-in refraction and rim treatment on top of blurred backdrop sampling.
This is the first proof target intended to read as liquid glass rather than
only a blurred translucent panel.

## Decisions

- Add a dedicated Studio runnable named
  `makepad-example-aichat-shader-backdrop-refraction-proof`.
- Use `AICHAT_GLASS_BACKEND=shader-backdrop-refraction-proof` for this
  diagnostic target.
- Reuse the Step 24 blurred backdrop texture as the sampling source.
- Add refraction as an opt-in `GlassPanel` shader parameter with a default of
  zero so existing panels remain unchanged.
- Add a subtle edge/rim boost as an opt-in `GlassPanel` shader parameter with a
  default of zero.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-25-shader-backdrop-refraction-proof.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not make refraction the default backend.
- Do not alter Apple native underlay behavior.
- Do not add native hit-test or multi-layer renderer changes.
- Do not remove Step 22-24 proof targets.

## Out of Scope

- Physically correct ray tracing.
- Real OS desktop sampling.
- Native Apple interleave implementation.
- Runtime quality controls.

## Acceptance Criteria

Scenario: refraction backend parses explicitly
Test: `cargo test -p makepad-example-aichat shader_backdrop_refraction_proof -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-refraction-proof`
When aichat resolves startup glass appearance
Then the appearance stays on shader substrate and enables refraction proof config

Scenario: Studio exposes refraction proof runnable
Test: `rg "makepad-example-aichat-shader-backdrop-refraction-proof|shader-backdrop-refraction-proof" makepad.splash examples/aichat/src/main.rs`
Given Studio lists aichat runnables
When the refraction proof target is launched
Then it requests `AICHAT_GLASS_BACKEND=shader-backdrop-refraction-proof`

Scenario: GlassPanel exposes opt-in refraction controls
Test: `rg "backdrop_refraction_strength|backdrop_rim_strength|backdrop_refracted_uv" widgets/src/glass_panel.rs`
Given existing `GlassPanel` call sites do not opt in
When the shader is inspected
Then refraction and rim parameters default to zero and do not change existing behavior

Scenario: refraction proof wires nonzero shader parameters
Test: `rg "Refraction|backdrop_refraction_strength|backdrop_rim_strength" examples/aichat/src/main.rs widgets/src/glass_panel.rs`
Given the refraction proof is active
When glass appearance is applied
Then aichat enables nonzero refraction and rim parameters on proof panels

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given refraction shader parameters and wiring are added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the proof is a Makepad shader-only step
When the git diff is inspected
Then no files under `platform/src` are listed
