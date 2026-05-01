# Plan: aichat liquid glass v3 shader backdrop

Supersedes the shader-rendered parts of [aichat-liquid-glass-v3.plan.md](aichat-liquid-glass-v3.plan.md).

This plan is the cross-platform real blur/refraction path. It must stay independent from macOS `NSGlassEffectView`; it should not touch `platform/src/`.

## Goal

Make ShaderOnly mode more like real glass by rendering a backdrop scene into textures, blurring it, and letting `GlassPanel` sample the blurred result with optional refraction.

Target platforms:

- macOS shader mode
- Linux
- Windows
- other Makepad render targets with texture-backed passes

Non-goal:

- sampling the real desktop behind the native window
- using `NSVisualEffectView` or `NSGlassEffectView`
- replacing the macOS native substrate path

## Architecture

```text
Pass A: backdrop_scene_pass
  -> backdrop_raw: Texture(RenderBGRAu8)

Pass B1: blur_h_pass
  samples backdrop_raw
  -> backdrop_blur_h

Pass B2: blur_v_pass
  samples backdrop_blur_h
  -> backdrop_blurred

Pass C: main window pass
  GlassPanel samples backdrop_blurred with panel-local UV mapping
```

The scene pass is an app-controlled artificial scene, not OS desktop capture. The first implementation should use the existing aichat decorative scene vectors because they are already on brand and deterministic.

## Phase SB-1: Backdrop Scene

Add a reusable aichat backdrop scene that can render into an offscreen texture.

Recommended first scene:

- reuse the dormant `ChatSceneVector` / liquid background gradients from `examples/aichat/src/main.rs`
- render at a fixed internal size initially, then resize with the window after the pass chain is stable

Acceptance:

- a standalone offscreen texture is populated every frame or whenever the scene changes
- default UI still renders exactly as current ShaderOnly when the texture is not sampled

## Phase SB-2: Blur Passes

Add a small blur-pass helper around two separable passes:

- horizontal Gaussian/Kawase pass
- vertical Gaussian/Kawase pass

Initial implementation can use 13-tap separable Gaussian weights. Optimize only after screenshots prove the visual direction.

Acceptance:

- `backdrop_blurred` visibly differs from `backdrop_raw`
- pass size can be reduced for performance without changing the main UI layout
- no platform code changes

## Phase SB-3: GlassPanel Sampling

Extend `GlassPanel` only if existing shader params cannot express the feature.

Needed inputs:

- blurred backdrop texture
- panel-to-backdrop UV scale
- panel-to-backdrop UV offset
- refraction strength
- backdrop mix amount

First proof should only sample the blurred texture and mix it with the current tint. Do not add refraction until texture coordinate plumbing is proven.

Acceptance:

- panels show blurred backdrop content inside their rounded shape
- existing tint, border, highlight, noise, and halo controls still work
- without a backdrop texture, `GlassPanel` falls back to current ShaderDefault behavior

## Phase SB-4: Per-Panel UV Mapping

Each panel must sample the part of the blurred backdrop that corresponds to its screen rect.

Preferred approach:

```text
panel_to_backdrop_uv_scale / panel_to_backdrop_uv_offset as per-instance values
```

This is more explicit than relying on fragment coordinates and keeps panel motion/layout changes understandable.

Acceptance:

- moving or resizing a panel updates the sampled backdrop region
- two panels at different positions sample different backdrop areas

## Phase SB-5: Refraction and Rim

Once sampling is stable, add glass-specific distortion:

- SDF/superellipse distance field for panel shape
- edge-weighted lensing that pulls samples toward the panel center
- subtle rim/specular multiplier based on distance to edge

Acceptance:

- the panel reads as refractive, not only blurred
- text remains readable over bright and dark scenes
- `Glass` slider remains a readability/opacity control, not a blur-radius slider

## Validation

Use Studio release runs for visual verification.

Required checks:

- bright backdrop scene
- dark backdrop scene
- narrow window resize
- generated Splash content visible over panels
- `AICHAT_GLASS_BACKEND=shader` remains the default path

Static checks:

```text
cargo check -p makepad-example-aichat
cargo test -p makepad-example-aichat
cargo build -p makepad-example-aichat --release
```

## Relationship to macOS Native Glass

This path is not a fallback for `NSGlassEffectView` internals. It is the cross-platform substrate.

The macOS native path owns:

- runtime `NSGlassEffectView` lookup
- native view insertion below Makepad
- native substrate result events

ShaderBackdrop owns:

- offscreen scene texture
- blur passes
- `GlassPanel` texture sampling/refraction

The two may share `GlassPanelPreset` tuning, but their substrate implementations remain separate.
