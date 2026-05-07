spec: task
name: "AI Chat Liquid Glass Step 30 Shader Backdrop Liquid Warp"
tags: [makepad, aichat, liquid-glass, shader-backdrop, refraction, studio-validation]
---

## Intent

Address the visual issue where `shader-backdrop-interior` still reads as
visible grid/stripes without obvious refraction or liquid distortion. The
candidate should reduce diagnostic grid strength and add a visible liquid UV
warp to the backdrop sampling path.

## Decisions

- Add `liquid_warp_strength` to `ShaderBackdropVisualProfile`.
- Keep `InteriorNoChroma` as a no-chroma comparison, but preserve the same
  liquid warp as `Interior`.
- Keep `Refraction` proof stronger than `Interior`.
- Lower the `Interior` scene grid below the previous diagnostic level.
- Implement liquid warp in `GlassPanel` backdrop UV sampling, not as a new pass.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-30-shader-backdrop-liquid-warp.spec`
- `examples/aichat/src/main.rs`
- `widgets/src/glass_panel.rs`

### Forbidden

- Do not remove visual comparison/proof targets.
- Do not change Studio runnable names.
- Do not make shader-backdrop the default backend.
- Do not add platform/native renderer changes.

## Out of Scope

- Physically-based refraction.
- New offscreen passes.
- Runtime tuning UI.
- Native Apple material work.

## Acceptance Criteria

Scenario: interior profile enables liquid warp
Test: `cargo test -p makepad-example-aichat shader_backdrop_interior_visual_profile_uses_liquid_warp -- --nocapture`
Given `Interior`, `InteriorNoChroma`, `Refraction`, and default shader profiles
When their visual profiles are inspected
Then `Interior` has non-zero liquid warp, no-chroma keeps the same warp, refraction remains stronger, and default shader remains zero

Scenario: liquid warp is wired through profile application
Test: `rg "liquid_warp_strength|backdrop_liquid_warp_strength" examples/aichat/src/main.rs widgets/src/glass_panel.rs`
Given a shader-backdrop candidate is active
When glass appearance is applied
Then every glass panel receives the profile liquid warp strength

Scenario: glass panel UV path applies liquid distortion
Test: `rg "backdrop_refracted_uv|wave_a|wave_b|liquid_weight|backdrop_liquid_warp_strength" widgets/src/glass_panel.rs`
Given a glass panel samples the backdrop texture
When liquid warp is enabled
Then backdrop UVs include time-varying wave distortion in addition to edge pull

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given liquid warp sampling is added
When affected crates are checked
Then they compile without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given this is shader-backdrop tuning
When the git diff is inspected
Then no files under `platform/src` are listed
