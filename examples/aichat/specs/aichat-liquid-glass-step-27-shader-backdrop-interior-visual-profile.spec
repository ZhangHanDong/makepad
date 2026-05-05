spec: task
name: "AI Chat Liquid Glass Step 27 Shader Backdrop Interior Visual Profile"
tags: [makepad, aichat, liquid-glass, shader-backdrop, visual-tuning, studio-validation]
---

## Intent

Reduce the diagnostic-grid look of `shader-backdrop-interior` while keeping
the strong grid in the proof targets. The candidate backend should read more
like an interior glass material and less like a sampler/debug texture.

## Decisions

- Keep `shader-backdrop-refraction-proof` as the stronger diagnostic target.
- Give `ShaderBackdropProof::Interior` a distinct visual profile.
- Lower the offscreen scene grid strength for `Interior`.
- Keep `Interior` refraction and rim weaker than `Refraction`.
- Do not change the default `shader` backend.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-27-shader-backdrop-interior-visual-profile.spec`
- `examples/aichat/src/main.rs`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not remove diagnostic proof targets.
- Do not change Studio runnable names.
- Do not make `shader-backdrop-interior` the default backend.
- Do not add platform/native renderer changes.

## Out of Scope

- Final production default selection.
- New blur algorithms.
- Runtime sliders for tuning.
- Native Apple interleave implementation.

## Acceptance Criteria

Scenario: interior visual profile is less diagnostic than refraction proof
Test: `cargo test -p makepad-example-aichat shader_backdrop_interior_visual_profile -- --nocapture`
Given both `Interior` and `Refraction` use the full backdrop chain
When their visual profiles are inspected
Then `Interior` has lower scene grid, refraction, and rim strengths than `Refraction`

Scenario: interior scene shader exposes grid strength
Test: `rg "scene_grid_strength|DrawAichatBackdropScene" examples/aichat/src/main.rs`
Given the offscreen scene is rendered
When the scene shader is inspected
Then the grid strength is controlled by a runtime uniform

Scenario: interior still uses the full backdrop chain
Test: `rg "ShaderBackdropProof::Interior|shader_backdrop_visual_profile|render_shader_backdrop_scene_pass" examples/aichat/src/main.rs`
Given the interior backend is active
When the routing code is inspected
Then it still renders the offscreen scene and uses blur/refraction sampling

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given the visual profile is added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the change is shader-only tuning
When the git diff is inspected
Then no files under `platform/src` are listed
