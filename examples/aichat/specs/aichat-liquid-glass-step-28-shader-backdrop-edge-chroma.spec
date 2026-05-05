spec: task
name: "AI Chat Liquid Glass Step 28 Shader Backdrop Edge Chroma"
tags: [makepad, aichat, liquid-glass, shader-backdrop, chroma, studio-validation]
---

## Intent

Make `shader-backdrop-interior` read more like glass without bringing back the
full diagnostic grid. The next visible cue is subtle edge chromatic dispersion:
RGB channels sample slightly different backdrop positions near the glass edge.

## Decisions

- Keep `shader-backdrop-interior` quieter than `shader-backdrop-refraction-proof`.
- Add chroma as part of `ShaderBackdropVisualProfile`.
- Apply chroma only through backdrop texture sampling in `GlassPanel`.
- Make chroma edge-weighted so the panel interior stays calm.
- Keep ordinary shader/default backends at zero chroma.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-28-shader-backdrop-edge-chroma.spec`
- `examples/aichat/src/main.rs`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not remove proof targets.
- Do not make `shader-backdrop-interior` the default backend.
- Do not change Studio runnable names.
- Do not add platform/native renderer changes.

## Out of Scope

- Full physically-based glass.
- Runtime tuning controls.
- Native Apple material interleaving.
- New render passes.

## Acceptance Criteria

Scenario: interior profile enables subtle chroma
Test: `cargo test -p makepad-example-aichat shader_backdrop_interior_visual_profile_uses_subtle_chroma -- --nocapture`
Given both `Interior` and `Refraction` use the full backdrop chain
When their visual profiles are inspected
Then `Interior` has non-zero chroma, lower chroma than `Refraction`, and the default shader profile remains zero

Scenario: chroma is wired through profile application
Test: `rg "chroma_strength|shader_backdrop_visual_profile" examples/aichat/src/main.rs widgets/src/glass_panel.rs`
Given the interior backend is active
When glass appearance is applied
Then the profile chroma value is passed to each glass panel draw shader

Scenario: chroma is edge-weighted texture sampling
Test: `rg "backdrop_texture_rgb|backdrop_edge_weight|chroma_strength" widgets/src/glass_panel.rs`
Given the glass panel samples the blurred backdrop texture
When chroma is enabled
Then RGB channels sample slightly offset UVs weighted by glass edge proximity

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given chroma sampling is added
When the affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the change is shader-only tuning
When the git diff is inspected
Then no files under `platform/src` are listed
