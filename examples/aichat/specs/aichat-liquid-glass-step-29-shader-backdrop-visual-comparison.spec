spec: task
name: "AI Chat Liquid Glass Step 29 Shader Backdrop Visual Comparison"
tags: [makepad, aichat, liquid-glass, shader-backdrop, visual-validation, studio-validation]
---

## Intent

Add a direct visual comparison path for the shader-backdrop glass candidate.
The user should be able to run three Studio items and compare the same UI with
no chroma, subtle chroma, and the stronger refraction proof.

## Decisions

- Add `shader-backdrop-interior-no-chroma` as a comparison backend.
- Keep `shader-backdrop-interior` as the current product-like candidate.
- Keep `shader-backdrop-refraction-proof` as the strong diagnostic comparison.
- Do not change the default `shader` backend.
- Do not touch `platform/src/**`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-29-shader-backdrop-visual-comparison.spec`
- `examples/aichat/src/main.rs`
- `makepad.splash`

### Forbidden

- Do not remove existing proof targets.
- Do not rename existing Studio runnable items.
- Do not make any shader-backdrop target the default backend.
- Do not add platform/native renderer changes.

## Out of Scope

- In-app runtime toggle UI.
- Production default selection.
- Native Apple material work.
- New render passes.

## Acceptance Criteria

Scenario: no-chroma comparison backend resolves
Test: `cargo test -p makepad-example-aichat shader_backdrop_interior_no_chroma -- --nocapture`
Given `AICHAT_GLASS_BACKEND=shader-backdrop-interior-no-chroma`
When startup glass appearance is resolved
Then it uses `ShaderBackdropProof::InteriorNoChroma`

Scenario: no-chroma profile preserves the interior chain
Test: `cargo test -p makepad-example-aichat shader_backdrop_interior_visual_profile_uses_subtle_chroma -- --nocapture`
Given `InteriorNoChroma` is compared with `Interior`
When their visual profiles are inspected
Then `InteriorNoChroma` keeps the same refraction and rim strengths but has zero chroma

Scenario: Studio exposes the comparison runnable
Test: `rg "makepad-example-aichat-shader-backdrop-interior-no-chroma|shader-backdrop-interior-no-chroma" makepad.splash examples/aichat/src/main.rs`
Given Studio run items are loaded
When a visual comparison is needed
Then a runnable exists for the no-chroma comparison backend

Scenario: affected app compiles
Test: `cargo check -p makepad-example-aichat`
Given the comparison backend is added
When the app crate is checked
Then it compiles without errors

Scenario: platform backend remains untouched
Test: `git diff --name-only HEAD -- platform/src`
Given the change is app-level comparison routing
When the git diff is inspected
Then no files under `platform/src` are listed
