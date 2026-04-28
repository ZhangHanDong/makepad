# Plan: aichat liquid-glass v3 — real backdrop blur + refraction

Supersedes: [aichat-liquid-glass-v2.plan.md](aichat-liquid-glass-v2.plan.md)
Issue context: [aichat-liquid-glass-not-real-glass.md](../issues/aichat-liquid-glass-not-real-glass.md), [aichat-liquid-glass-backdrop-platform-bug.md](../issues/aichat-liquid-glass-backdrop-platform-bug.md)
References:
- https://kube.io/blog/liquid-glass-css-svg/ — CSS/SVG `feDisplacementMap` + `feGaussianBlur`
- https://github.com/OverShifted/LiquidGlass — OpenGL implementation, the canonical recipe

## Why v2 fell short

v2 leaned on OS transparency + a per-panel SDF shader with halo + tint. After review against the two reference implementations, the gap is structural — v2 has **none** of the three real liquid-glass ingredients:

| Ingredient | v2 | What it requires |
|---|---|---|
| Backdrop blur | ❌ Crisp through-glass; `use_scene_blur` was a fake constant lerp | Render scene → 2× separable Gaussian → sample blurred texture |
| Refraction displacement | ❌ Pixels sample own-coords or nothing | SDF-driven lensing: edge pixels sample from closer to center |
| Specular rim | 🟡 SDF outer halo (additive cyan) | Angle-modulated multiplier on the sampled color, weighted by SDF distance from edge |

Without all three, panels read as "tinted translucent rectangles", not glass. That's exactly what the user called out.

## v3 architecture

Three-pass rendering, all in widget layer (Makepad already has the infra — verified):

```
Pass A: scene_pass         (offscreen FBO, clears to backdrop scene)
   ↓ writes to backdrop_raw : Texture (RenderBGRAu8)
Pass B1: blur_h_pass        (offscreen FBO, samples backdrop_raw, runs 13-tap horizontal Gaussian)
   ↓ writes to backdrop_blur_h : Texture
Pass B2: blur_v_pass        (offscreen FBO, samples backdrop_blur_h, runs 13-tap vertical Gaussian)
   ↓ writes to backdrop_blurred : Texture
Pass C: main window pass    (composites UI tree onto screen)
   GlassPanel.draw_bg samples backdrop_blurred with displaced UV
```

Existing Makepad APIs:
- `Texture::new_with_format(cx, TextureFormat::RenderBGRAu8 { size, initial: ... })`
- `DrawPass::set_color_texture(cx, texture, slot)` — make pass render to texture
- `DrawPassMode::Texture` — already wired across macOS / iOS / tvOS / Linux / Windows
- `draw_vars.texture_slots[i] = Some(texture)` — bind texture into a shader's sampler
- Existing examples that already do offscreen rendering: `examples/exf`, `examples/arracing`

So **no platform changes needed**. v3 can ship without touching `platform/src/` at all.

## Phase plan

### Phase 1: scaffolding — backdrop scene pass + blur passes

Critical files:
- `widgets/src/glass_panel.rs` (rewrite — full liquid-glass shader)
- `widgets/src/blur_pass.rs` (new — encapsulates the two-direction Gaussian)
- `examples/aichat/src/main.rs` (wire scene + blur → expose blurred texture as `theme.texture_backdrop_blur`)

Cost: medium. The blur shader is straightforward (13-tap separable, weights from `Blur.glsl` lifted as-is). The scaffolding is mostly `Texture` + `DrawPass` setup, mirroring `examples/exf`.

### Phase 2: GlassPanel shader rewrite — refraction + glow

Replace v2's shader entirely. New shader:

```glsl
// Per-panel instance params:
//   tint_color, tint_alpha, corner_radius
//   glass_curve (u_powerFactor, controls superellipse n)
//   refraction_strength (controls how aggressively edges bend)
//   glow_strength, glow_edge0, glow_edge1
//   noise_strength
// Plus uniforms (per pass):
//   sampler2D u_backdrop_blurred
//   vec2      u_panel_to_backdrop_uv_scale
//   vec2      u_panel_to_backdrop_uv_offset

float sdSuperellipse(vec2 p, float n, float r) {
    vec2 a = abs(p);
    float num = pow(a.x, n) + pow(a.y, n) - pow(r, n);
    float den = n * sqrt(pow(a.x, 2.0*n - 2.0) + pow(a.y, 2.0*n - 2.0)) + 1e-5;
    return num / den;
}

float lensing(float dist) {
    // f(d) from OverShifted: edges (d≈0) pull strongly toward center;
    //                       interior (d≈1) untouched.
    // Tuned constants land near the reference values; expose as instance.
    float a = 0.7, b = 2.3, c = 5.2, d = 6.9;
    return 1.0 - b * pow(c * 2.71828, -d * dist - a);
}

pixel: fn() {
    let p = (self.pos - 0.5) * 2.0
    let d = sdSuperellipse(p, self.glass_curve, 1.0)
    if d > 0.0 { discard }
    let dist = -d

    // Refract: pixels near edge sample from closer to panel center
    let displaced = p * pow(lensing(dist), self.refraction_strength)

    // Map displaced panel-local coord → backdrop UV
    let backdrop_uv = displaced * self.u_panel_to_backdrop_uv_scale + self.u_panel_to_backdrop_uv_offset

    // Sample the pre-blurred backdrop
    let bg = texture2d(self.u_backdrop_blurred, backdrop_uv)

    // Glow / specular rim
    let angle = atan(p.y, p.x)
    let rim = sin(angle - 0.5) * self.glow_strength
            * smoothstep(self.glow_edge0, self.glow_edge1, dist) + 1.0

    // Composite: tint over refracted backdrop, modulated by rim, plus noise
    let noise = (Math.random_2d(self.pos * self.rect_size + vec2(self.draw_pass.time * 37.0, self.draw_pass.time * 13.0)) - 0.5) * self.noise_strength
    let glass_rgb = mix(bg.rgb, self.tint_color.rgb, self.tint_alpha) * rim + noise
    return vec4(glass_rgb, 1.0)
}
```

Cost: medium. The math is documented; tuning the constants is the work.

### Phase 3: backdrop scene

Decision: what gets rendered in Pass A?

| Option | Tradeoff |
|---|---|
| **A1.** Reuse the dormant `ChatSceneVector` (gradient + decorative orbs) at `main.rs:101-124` | Already drawn; just instantiate it into the scene pass. Animatable. Doesn't show desktop wallpaper. |
| **A2.** Bundled wallpaper image (e.g., `resources/backdrop-mountain.jpg`) | Matches the reference mockup look. Static. |
| **A3.** Capture screen behind window via macOS `CGWindowListCreateImage` once per second, upload as texture | Real desktop transmission. Expensive, platform-specific, doesn't update during animation. |

Recommendation: **A1 first**, then A2 as user-selectable theme. A3 is v4 if the user really wants real desktop bleed-through.

### Phase 4: per-panel UV mapping

Each `GlassPanel` is rendered at a different position in the window. Its shader needs to know which region of the blurred backdrop to sample. Two approaches:

| Approach | How |
|---|---|
| **B1.** Pass `panel_to_backdrop_uv_scale/offset` per panel as instance | Computed Rust-side from panel's screen rect; one set per panel. Clean, what OverShifted does. |
| **B2.** Use `draw_pass.fragment_coord` (gl_FragCoord) divided by viewport size | Simpler but couples shader to viewport. |

Recommendation: **B1**. It's cleaner and lets us animate panel motion correctly.

### Phase 5: tune + verify

- Per-panel tint_alpha can drop back from v2's 0.86–0.94 to **0.30–0.55** — with real blur, low alpha works again
- Sweep `Glass` slider: visible blur intensity should change subtly (could map slider to `refraction_strength` or `tint_alpha`, not blur radius itself which is per-pass uniform)
- Screenshot over the user's mountain wallpaper should match the 4/27 reference

## Concrete first commit (Phase 1 only, ~half day)

1. Add `ChatScene` View that renders `ChatSceneVector` from `main.rs:101-124` (currently dead code) into a 1024×1024 offscreen texture
2. Add `widgets/src/blur_pass.rs` with two passes (horizontal, vertical) using 13-tap weights from `Blur.glsl`
3. Wire chain: `ChatScene` pass output → blur_h pass input → blur_v pass output → `theme.texture_backdrop_blurred`
4. Update `GlassPanel` to take a `texture: texture2d` slot bound to `theme.texture_backdrop_blurred`
5. Initial pixel function: just sample the blurred texture at `self.pos` and mix with tint — no refraction yet, just to prove the plumbing works

Verification at end of Phase 1: aichat shell shows **blurred ChatSceneVector** (decorative orbs/gradient) inside the panels, confirming the multi-pass chain is wired. No refraction yet — that's Phase 2.

## What's explicitly deferred to v4

- A3 real desktop capture (CGWindowListCreateImage)
- Animated halo / breathing
- Light theme glass
- iridescent / chromatic dispersion (currently `chroma_strength` is a no-op instance from v2)

## Out of band: the platform bug

`addSubview:positioned: 0i64` in `platform/src/os/apple/macos/macos_window.rs:532` still crashes on macOS 15.5. v3 doesn't need NSVisualEffectView so the bug doesn't block v3, but it should still be fixed in a separate PR (one-line `0i64` → `-1i64`). Issue tracked at [aichat-liquid-glass-backdrop-platform-bug.md](../issues/aichat-liquid-glass-backdrop-platform-bug.md).

## Rollback

v3 changes are confined to:
- `widgets/src/glass_panel.rs` (rewrite)
- `widgets/src/blur_pass.rs` (new)
- `widgets/src/lib.rs` (export blur_pass module)
- `examples/aichat/src/main.rs` (instantiate scene + blur passes; bind blurred texture to glass panels)

Reverting these files restores v2. No platform code touched.
