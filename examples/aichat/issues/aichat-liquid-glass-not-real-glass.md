# Issue: aichat liquid-glass UI looks opaque, not glass

Status: open
Severity: visual / spec compliance
Spec: [examples/aichat/specs/aichat-liquid-glass-ui.spec.md](../specs/aichat-liquid-glass-ui.spec.md)
Reference image: 4/24 ChatGPT mountain-wallpaper mock + 4/27 mountain-wallpaper redesign mock
Owner: aichat
Discovered: 2026-04-27 via `/screenshot` verification of running build

## Symptom

Running `makepad-example-aichat` and capturing the window over a non-trivial desktop produces a chat shell that looks like a flat dark green rectangle. Compared to the approved reference mocks, four visual properties are missing or wrong:

| Reference mock | Current build |
|---|---|
| Desktop wallpaper visibly透 through the whole shell, especially around the rounded outer edge and the right side of the main area | Shell appears solid; sidebar in particular is fully opaque |
| Each glass layer (shell / sidebar / main / card / composer) tints **a different bit of the wallpaper**, giving a stack of frosted layers | All layers look like the same flat green; no per-layer transparency variation |
| Outer cyan rim halo extending **outside** the panel edge, plus a soft top highlight band on each panel | Only an inset 1px cyan border line; no glow, no top highlight |
| Subtle chromatic dispersion at panel edges (slight prism-like color separation) | None |

Spec acceptance criteria affected:
- `aichat_liquid_glass_shell_contract` — "shell has a cyan/blue glass border or glow" + "shell background opacity is between 0.85 and 0.92" — current shell does not show desktop通过, opacity is effectively higher than 0.92 because **inner layers stack on top of an already 0.90-alpha shell**, compounding to near-1.0 transmittance loss
- `aichat_output_card_readability_contract` — "Given the content is rendered over a bright or detailed desktop wallpaper" — wallpaper is not visible, so this scenario can't even be triggered
- `aichat_glass_opacity_slider_contract` — slider exists and updates the percentage label, but the visible difference at 0.85 vs 0.92 is barely perceptible because the underlying alpha math caps the visible range

## Root cause

Two independent problems compound:

### 1. Window is transparent at the OS layer, but panels are nearly opaque

`examples/aichat/src/main.rs:467-498`:

```
main_window := Window{
    show_caption_bar: false
    pass.clear_color: #00000000        // OK — clears to transparent
    window.transparent: true           // OK — NSWindow.isOpaque = NO
    window.macos: MacosWindowConfig{chrome: MacosWindowChrome.Borderless}
    ...
    app_shell := GlassPanel {
        ...
        tint_alpha: 0.90               // 10% see-through
    }
    sidebar := GlassPanel {
        tint_alpha: 1.0                // 0% see-through — fully opaque
    }
    main_area := GlassPanel {
        tint_alpha: 0.96               // 4% see-through
    }
    composer := GlassPanel {
        tint_alpha: 0.98               // 2% see-through
    }
```

The window-level transparency is wired up correctly (verified `platform/src/os/apple/macos/macos.rs:887-888` toggles `layer_opaque = NO` and `layer_alpha = 0.0` when `visuals.transparent` is true). But every panel's `tint_alpha` is so high that the OS-level transparency is wasted. **Sidebar at 1.0 will never show desktop content regardless of OS opacity**.

### 2. `GlassPanel` shader is fake glass

`widgets/src/glass_panel.rs:23-58` — the `pixel` function. Three things are missing:

#### (a) `use_scene_blur` is a misnomer

```rust
let blur_mix = clamp(self.use_scene_blur * self.blur_amount, 0.0, 1.0)
let blur_fallback_color = vec3(0.86, 0.9, 0.96)         // hard-coded constant
let glass_color = self.tint_color.rgb.mix(blur_fallback_color, blur_mix * 0.45)
```

There is no scene texture sample. `use_scene_blur` just lerps the tint toward a fixed pale blue-grey. Every pixel of every panel computes the **same** "blurred backdrop", so panels can't show different bits of wallpaper through them. The name promises a scene sample, the code delivers a constant.

#### (b) No external rim halo

Halos / glow rings live **outside** the SDF surface, computed on negative-distance pixels. The current shader fills the SDF and then strokes a 1px border with `border_alpha`. There is no halo extending past the panel edge.

#### (c) No top highlight band

The shader has `specular_strength`, but it's used like this:

```rust
let edge_uv = abs(self.pos * 2.0 - 1.0)
let edge_gradient = clamp((edge_uv.x + edge_uv.y) * 0.5, 0.0, 1.0)
let highlight = self.specular_strength * (0.65 * edge_gradient + 0.35 * (1.0 - self.pos.y))
```

This produces a diagonal-from-top-left shimmer, not the **horizontal top-edge highlight band** that real glass panels have (think of the thin bright line on macOS Big Sur's frosted bars). The current "highlight" reads as a faint diagonal noise rather than a glass top.

### 3. Dead code amplifies confusion

`main.rs:101-124` declares a `ChatSceneVector` (gradient + decorative orbs + curves) but it is never instantiated in the UI tree. The spec author probably intended this as the "scene" to be sampled by `GlassPanel`'s blur, but the wiring was never finished. Reading the code suggests a backdrop pass exists; running the binary proves it doesn't.

## Why the OS opacity already does most of what's needed

A common false alarm here is "we need a backdrop blur pass like NSVisualEffectView". For the v1 visual we don't:

- **Desktop content already lives behind the metal layer.** When `window.transparent: true`, the macOS compositor draws the desktop / wallpaper / windows behind aichat into the framebuffer. As long as panel alpha < 1.0, OS compositing shows it through. No app-side backdrop pass is required for **transmittance** alone.
- **What we lose without a blur pass**: sharpness — desktop edges show through crisply rather than frosted. This is acceptable for v1; real backdrop blur is parked as a future enhancement (NSVisualEffectView injection or a Pass-based separable Gaussian).
- **What we gain**: per-layer透出 differs immediately because each layer's alpha is different and each samples a different region of the framebuffer behind it.

## Recommended solution (4 layers, ranked by ROI)

| # | Change | File | Cost | Visual gain |
|---|---|---|---|---|
| 1 | Lower per-panel `tint_alpha` (shell ~0.55, sidebar ~0.72, main ~0.62, card ~0.82, composer ~0.78) | `examples/aichat/src/main.rs` | trivial | ★★★ desktop透出立刻显形 |
| 2 | Rewrite `GlassPanel` shader: top highlight band + external rim halo + chromatic edge | `widgets/src/glass_panel.rs` | medium | ★★★ glass shape stops looking like a colored rectangle |
| 3 | Per-layer halo / border / highlight tuning | `examples/aichat/src/main.rs` | low | ★★ depth stack reads correctly |
| 4 | Real backdrop blur (NSVisualEffectView injection on macOS, framebuffer pass blur on other platforms) | `platform/src/os/apple/...` + new `Pass` | high | ★★ frosting; deferred |

`#1 + #2 + #3` together close ~85% of the gap and are scoped to two files. `#4` is genuine new infrastructure and can ship later without breaking the spec.

## Verification

Acceptance: re-run `/screenshot` after applying #1–#3 and confirm against the redesign reference image:
- Mountain wallpaper visible through every panel
- Each layer's tint distinguishably different (shell darkest, card brightest)
- Cyan glow ring visible outside shell rounded edge
- Thin horizontal top highlight on each panel
- `Glass` slider's 85%↔92% sweep produces a clearly visible alpha change

Re-running existing acceptance scenarios:
- `aichat_liquid_glass_shell_contract` should now pass on the "cyan/blue glass border or glow" and "opacity 0.85–0.92" clauses
- `aichat_output_card_readability_contract` becomes meaningfully testable

## Related artifacts

- Redesign visual spec: [aichat-liquid-glass-v2.spec.md](../specs/aichat-liquid-glass-v2.spec.md)
- Redesign implementation plan: [aichat-liquid-glass-v2.plan.md](../specs/aichat-liquid-glass-v2.plan.md)
- Reference mockups: 4/24 + 4/27 ChatGPT mountain-wallpaper mocks (provided by user, not in repo)
