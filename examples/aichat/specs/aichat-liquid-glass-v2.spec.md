spec: task
name: "AI Chat Liquid Glass UI v2"
tags: [makepad, aichat, ui, liquid-glass, v2]
supersedes: aichat-liquid-glass-ui.spec.md
---

## Intent

Refine `makepad-example-aichat` so it actually looks like the approved 4/27
mountain-wallpaper reference: a desktop-transparent emerald liquid-glass shell
where the wallpaper is **visibly seen through every layer**, with a stack of
distinguishable glass panels (shell / sidebar / main area / cards / composer),
a cyan rim halo around the outer shell, and per-panel top highlight bands.

v1 ([aichat-liquid-glass-ui.spec.md](aichat-liquid-glass-ui.spec.md)) defined
the layout and color language. v1 is achieved structurally but **not optically**
— the panels are alpha 0.90+ and the shader produces a flat tint, not glass.
v2 supersedes v1 on visual decisions; v1 layout / sidebar / composer /
acceptance scenarios remain in force unless explicitly overridden below.

## What changed vs v1

- **Default panel alpha is no longer a single 0.85–0.92 number.** Different
  layers must have **different** alphas so the depth stack reads. The `Glass`
  slider scales them in proportion, not uniformly to a single value.
- **The shell must show desktop wallpaper通过 it**, not just appear semi-tinted.
- **Each panel must have a top highlight band** (a thin horizontal bright line
  near the top edge), not a diagonal specular gradient.
- **The outer shell must have a cyan halo extending outside its rounded edge**,
  not just an inset 1px border line.
- **Edges may have subtle chromatic dispersion** (slight prism-like color
  separation at panel borders). Optional but encouraged.
- **Backdrop blur is deferred.** v2 accepts crisp-through-glass desktop. A
  later v3 may add NSVisualEffectView / Pass-based backdrop blur.

## Decisions

### Per-layer alpha targets (at Glass slider = 90%, the default)

| Layer | `tint_alpha` | Tint color | Border / halo | Top highlight |
|---|---|---|---|---|
| `app_shell` (outer rounded frame) | **0.70** | deep emerald `ai_panel` | cyan border 0.55 + cyan rim halo extending ~14px outside the SDF edge, strength 0.55 | none — shell is just frame |
| `sidebar` | **0.85** | deeper ink emerald `#x021310` | cyan inset border 0.30, no halo | thin top highlight band, 0.18 |
| `main_area` | **0.80** | mid emerald `#x031C16` | cyan inset border 0.18 | thin top highlight band, 0.14 |
| `output_card` (prose) | **0.82** | dark emerald `#x05221B` | cream inset border 0.30 | thin top highlight band, 0.20 |
| `output_card` (code / diagram) | **0.92** | very dark ink `#x021511` | cyan inset border 0.20 | thin top highlight band, 0.16 |
| `composer` | **0.88** | dark emerald `ai_panel` | cyan inset border 0.40 + outer halo strength 0.25, radius 10 | thin top highlight band, 0.22 |

The first v2 draft set shell/main/sidebar/composer to 0.55/0.62/0.72/0.78. Visual
review showed those let other windows' content read **through** the glass —
wallpaper is fine, content competition is not. The values above keep wallpaper
visible as texture while keeping every layer of UI dominant for foreground reading.

The `Glass` slider maps `[0%, 100%]` onto a per-layer multiplier curve:
`effective_alpha = base_alpha * (0.55 + slider * 0.45)`. So at slider = 0% the
shell goes to ~0.30 (ghost) and at 100% goes to ~1.00 (opaque). At default 90%
it produces the per-layer values in the table above.

### Cyan rim halo (outer shell)

- The halo is computed in the shader **outside** the SDF surface (negative SDF
  distance pixels).
- Soft falloff: `halo = smoothstep(halo_radius, 0.0, abs(sdf_dist)) * cyan_color * halo_strength`
  where `halo_radius` is in physical pixels (target ~12px at standard DPI).
- Halo color is the same cyan family as `theme.color_chat_cyan` (`#x72E4FF`).
- Halo only renders for the outer shell. Other panels may opt in via a
  `halo_strength` instance, default 0.0.

### Top highlight band

- A thin horizontal band of brightness at the panel's top edge.
- Function: `band = smoothstep(band_height, 0.0, self.pos.y * self.rect_size.y) * highlight_strength`
  with `band_height` ~3 physical pixels.
- Color: light cream `#xEAD8B8` blended at panel-specific `highlight_strength`.
- Replaces the v1 diagonal `specular_strength` term.

### Edge chromatic dispersion (optional)

- Near the SDF border (within ~2px of edge), allow R / G / B channels of the
  fill color to offset by ±0.5px each. Produces a subtle prism look.
- `chroma_strength` instance, default 0.0. Enable on shell + composer at 0.5.

### Window-level (already correct, must not regress)

- `window.transparent: true`
- `pass.clear_color: #00000000`
- `MacosWindowChrome.Borderless`
- `show_caption_bar: false`
- These are preconditions, not new decisions. Tests must guard them so future
  edits don't accidentally reset.

### What is explicitly not in v2

- **No backdrop blur.** Desktop transmits sharply. Frosted blur is v3.
- **No iridescent / animated highlight sweeps.** Static glass.
- **No new compositing pass.** All effects fit inside the existing `GlassPanel`
  shader and per-instance parameters.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/**`
- `examples/aichat/issues/**`
- `widgets/src/glass_panel.rs`
- `widgets/src/lib.rs` (only if a new instance parameter requires re-export)

### Forbidden

- Do not reintroduce `tint_alpha: 1.0` on any visible panel.
- Do not remove `window.transparent: true` or `pass.clear_color: #00000000`.
- Do not add a webview, HTML/CSS renderer, or external GUI framework.
- Do not add a new Pass / framebuffer / NSVisualEffectView in v2 (deferred to v3).
- Do not change existing chat backend selection behavior.
- Do not break v1 acceptance scenarios that aren't explicitly superseded here.

### Out of Scope

- Real backdrop blur (NSVisualEffectView / Pass-based Gaussian). Deferred.
- Animated halo / breathing glow.
- Per-monitor halo radius scaling.
- Theme-switching (light glass).

## Acceptance Criteria (additive to v1)

Scenario: Desktop wallpaper is visible通过 the shell
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_desktop_visible_through_shell
  Given the AI Chat window is launched over a desktop with a high-contrast wallpaper
  When the rendered window is captured and sampled
  Then at least 30% of pixels inside the shell rounded rect have non-trivial
       hue variance derived from the desktop behind the window
  And the variance is not explained by the panel tint color alone

Scenario: Glass layers stack with distinguishable alphas
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_layer_alpha_stack
  Given the default Glass slider value (90%)
  When the rendered window is sampled at one pixel inside the sidebar interior,
       one pixel inside the main area interior, and one pixel inside an output
       card interior
  Then the three sampled luminance values differ pairwise by at least 0.04
  And the ordering is `shell < main_area < sidebar < card` (ascending opacity)

Scenario: Outer shell has a cyan rim halo
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_shell_rim_halo
  Given the shell is rendered
  When the pixels in a 12-pixel band immediately outside the shell rounded
       edge are sampled
  Then the average chromaticity is shifted toward cyan
  And the average alpha falls off monotonically from inside to outside the band

Scenario: Each panel has a top highlight band
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_top_highlight_band
  Given a panel is rendered (sidebar, main area, card, composer)
  When the pixel row 1 below the panel's rounded top edge is sampled
  Then the row's average luminance is at least 0.05 higher than the row 8
       pixels below it

Scenario: Glass slider scales per-layer alpha non-uniformly
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_slider_scales_per_layer
  Given the slider is at 30%
  When the same three sample points from the layer-stack scenario are sampled
  Then all three luminance values are lower than at slider 90%
  And the relative ordering shell < main < sidebar < card is preserved
  And no panel becomes alpha 0 or alpha 1 at any slider position in [10%, 100%]

Scenario: Window-level transparency must not regress
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_window_transparency_preserved
  Given the script_mod source for `main_window`
  When parsed
  Then `window.transparent` is `true`
  And `pass.clear_color` is `#00000000`
  And the macOS chrome is `Borderless`
  And `show_caption_bar` is `false`

Scenario: v1 acceptance criteria still pass
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v1_regression_suite
  Given v2 changes are applied
  When the v1 acceptance criteria are re-run (sidebar, top toolbar, composer,
       horizontal scroll, min window layout, backend behavior, network failure)
  Then all of them still pass without modification

Scenario: Backdrop blur is intentionally absent
  Test:
    Package: makepad-example-aichat
    Filter: aichat_liquid_glass_v2_no_backdrop_blur_yet
  Given v2 is in effect
  When the GlassPanel shader source is inspected
  Then it does not sample any backdrop / scene texture
  And no new Pass with a Gaussian / box / kawase blur step is registered
  And this is by design and is documented as a v3 deferred item
