# Plan: aichat liquid-glass v2 implementation

For [aichat-liquid-glass-v2.spec.md](aichat-liquid-glass-v2.spec.md). Issue
context: [aichat-liquid-glass-not-real-glass.md](../issues/aichat-liquid-glass-not-real-glass.md).

## Context

The current build sets `window.transparent: true` correctly, but every panel's
`tint_alpha` is between 0.90 and 1.00, so the OS-level transparency is wasted
and the desktop wallpaper never shows through. The `GlassPanel` shader has no
external rim halo, no top highlight band, and a fake "scene blur" that lerps
to a constant pale blue-grey. v2 fixes both ends without introducing a new
compositing pass.

Cost summary:

| Phase | Files | Effort | Risk |
|---|---|---|---|
| 1. Per-layer alpha rebalance | `examples/aichat/src/main.rs` | trivial | low |
| 2. GlassPanel shader rewrite | `widgets/src/glass_panel.rs` | medium | low — additive instance params |
| 3. Per-panel halo / highlight tuning | `examples/aichat/src/main.rs` | low | low |
| 4. Slider scaling curve | `examples/aichat/src/main.rs` | trivial | low |

## Phase 1 — per-layer alpha rebalance

### Critical files
- `examples/aichat/src/main.rs`

### Current state
```rust
const DEFAULT_GLASS_OPACITY: f64 = 0.90;
const MIN_GLASS_OPACITY: f64 = 0.72;
const MAX_GLASS_OPACITY: f64 = 0.98;

fn glass_opacity_values(opacity: f64) -> GlassOpacity {
    let opacity = opacity.clamp(MIN_GLASS_OPACITY, MAX_GLASS_OPACITY);
    GlassOpacity {
        app: opacity as f32,
        sidebar: (opacity + 0.06).min(0.98) as f32,    // sidebar MORE opaque than shell — wrong direction
        main: (opacity + 0.04).min(0.97) as f32,
        composer: (opacity + 0.03).min(0.98) as f32,
    }
}
```

### Target state
```rust
const DEFAULT_GLASS_OPACITY: f64 = 0.90;        // slider UI value, range [0%, 100%]
const MIN_GLASS_OPACITY: f64 = 0.10;            // ghost — wallpaper dominant
const MAX_GLASS_OPACITY: f64 = 1.00;            // fully opaque

// Per-layer base alphas at slider = 90% (the v2 reference look).
// effective_alpha = base * (0.55 + slider * 0.45)
// At slider 0.90 → effective ≈ base. At slider 0.10 → effective ≈ base * 0.595.
// All layers stay strictly < 1.0 except by user explicit 100% slider.
struct GlassBase {
    shell: f32,    // 0.55 — outer frame, most see-through
    sidebar: f32,  // 0.72
    main: f32,     // 0.62
    card: f32,     // 0.82 (prose) — code/diagram cards override at site
    composer: f32, // 0.78
}

const GLASS_BASE: GlassBase = GlassBase {
    shell: 0.55, sidebar: 0.72, main: 0.62, card: 0.82, composer: 0.78,
};

fn glass_opacity_values(slider: f64) -> GlassOpacity {
    let s = slider.clamp(MIN_GLASS_OPACITY, MAX_GLASS_OPACITY);
    let scale = (0.55 + s * 0.45) as f32;
    GlassOpacity {
        app: (GLASS_BASE.shell * scale).min(1.0),
        sidebar: (GLASS_BASE.sidebar * scale).min(1.0),
        main: (GLASS_BASE.main * scale).min(1.0),
        composer: (GLASS_BASE.composer * scale).min(1.0),
    }
}
```

The existing `apply_glass_opacity` (`main.rs:2201-2229`) consumes `GlassOpacity`
unchanged. Only the field values shift. The `script_apply_eval!` block stays.

### Static panel declarations (`main.rs:480-518, 685-705, 828-846`)

Update at-rest values to match the v2 spec table:

| Site | line | `tint_color` | `tint_alpha` | `border_color` | `border_alpha` | `corner_radius` |
|---|---|---|---|---|---|---|
| `app_shell` | 480 | `ai_panel` | **0.55** | `ai_cyan` | 0.50 | 30 |
| `sidebar` | 500 | `#x021310` | **0.72** | `#x72E4FF` | 0.30 | 0 |
| `main_area` | 685 | `#x031C16` | **0.62** | `#x72E4FF` | 0.18 | 0 |
| `composer` | 828 | `ai_panel` | **0.78** | `ai_cyan` | 0.40 | 24 |

Output card alphas (prose / code / diagram) live in the markdown widget
template inside `ChatList`. They are `RoundedView { draw_bg +: { color: ... } }`
not `GlassPanel`. v2 keeps them as `RoundedView` for now — rebalance their
hex colors to match spec table (prose `#x05221BD2`, code `#x021511EB`).

### Tests
- Update existing test `glass_opacity_values_*` (`main.rs:2485-2497`) for the
  new clamp range and per-layer scaling. Assert ordering
  `shell < main < composer < sidebar < card` at any slider value in the legal
  range, monotonic in slider.
- New test verifying `glass_opacity_values(0.10).shell ≈ 0.55 * 0.595 ≈ 0.327`.

### Verification
- `cargo run -p makepad-example-aichat`
- `/screenshot` — desktop wallpaper visible through every panel; sidebar tone
  distinguishably warmer than main area when over a wallpaper with vertical
  color variation.

## Phase 2 — GlassPanel shader rewrite

### Critical files
- `widgets/src/glass_panel.rs`

### Current state (lines 7-60)
- `use_scene_blur` instance lerps tint to a hard-coded pale blue-grey.
- `specular_strength` produces a diagonal edge gradient.
- No external halo. No top highlight band. No chromatic edge.

### Target state

Add three new instance parameters; keep existing names for backwards compat
where possible; deprecate `use_scene_blur` and `blur_amount` (keep as no-op
to avoid breaking existing call sites in this commit, removed in a follow-up).

```rust
mod.widgets.GlassPanel = View{
    show_bg: true
    draw_bg +: {
        tint_color: instance(#fff)
        tint_alpha: instance(0.2)

        border_color: instance(#fff)
        border_alpha: instance(0.35)
        border_width: instance(1.0)
        corner_radius: instance(12.0)

        // NEW
        halo_color: instance(#x72E4FF)
        halo_strength: instance(0.0)         // 0 disables; shell uses 0.45
        halo_radius: instance(12.0)          // physical pixels outside SDF edge
        highlight_strength: instance(0.0)    // 0 disables; panels use 0.14-0.22
        highlight_band_height: instance(3.0) // physical pixels
        chroma_strength: instance(0.0)       // 0 disables; shell+composer 0.5
        noise_strength: instance(0.035)

        // DEPRECATED (no-op kept for compat — removed in follow-up commit)
        specular_strength: instance(0.0)
        use_scene_blur: instance(0.0)
        blur_amount: instance(0.0)

        pixel: fn() {
            let sdf = Sdf2d.viewport(self.pos * self.rect_size)
            let inset = self.border_width * 0.5
            sdf.box(
                inset, inset,
                self.rect_size.x - inset * 2.0,
                self.rect_size.y - inset * 2.0,
                self.corner_radius
            )

            // Top highlight band — bright row near y=0
            let y_px = self.pos.y * self.rect_size.y
            let band = clamp(1.0 - y_px / self.highlight_band_height, 0.0, 1.0)
            let highlight = vec3(1.0, 0.97, 0.86) * band * self.highlight_strength

            // Subtle animated noise to break flatness
            let noise = (
                Math.random_2d(
                    self.pos * self.rect_size
                    + vec2(self.draw_pass.time * 37.0, self.draw_pass.time * 13.0)
                ) - 0.5
            ) * self.noise_strength

            let fill_rgb = self.tint_color.rgb + highlight + noise
            let fill = vec4(fill_rgb, self.tint_alpha)
            sdf.fill_keep(fill)

            // Border stroke (kept)
            if self.border_width > 0.0 {
                sdf.stroke(
                    vec4(self.border_color.rgb, self.border_alpha),
                    self.border_width
                )
            }

            // External halo — only renders for negative SDF distance pixels
            // outside the panel edge. Falloff over halo_radius pixels.
            // Note: Sdf2d.result already premultiplied; we add halo on top.
            if self.halo_strength > 0.0 {
                let d = abs(sdf.dist)  // sdf.dist is signed; outside is positive
                let halo_a = clamp(
                    (1.0 - d / self.halo_radius)
                    * self.halo_strength,
                    0.0, 1.0
                )
                let halo_rgb = self.halo_color.rgb * halo_a
                let prev = sdf.result
                sdf.result = vec4(
                    prev.rgb + halo_rgb * (1.0 - prev.a),
                    prev.a + halo_a * (1.0 - prev.a)
                )
            }

            return sdf.result
        }
    }
}
```

### Notes / risks
- The exact `Sdf2d` field name for "signed distance" is `sdf.dist` in current
  Makepad shaders — verify by grepping `widgets/src` for `sdf.dist` or
  `sdf.last_pos` and use whichever the project consistently uses. If a public
  signed-distance accessor isn't exposed, halo can be approximated by fattening
  the SDF rect by `halo_radius` and rendering a separate translucent halo box
  underneath the panel before stroking. That fallback is still O(1) per pixel.
- Chromatic edge is **not** in the first cut of this shader — ship halo +
  highlight first, add chroma in a follow-up. Spec lists it as optional.

### Tests
- A new shader-output snapshot test under `widgets/tests/` (or extend an existing
  visual test if one exists) renders a 200×200 panel with `halo_strength=0.5`
  and verifies pixels outside the rounded rect have non-zero alpha and a cyan
  hue. If shader snapshot infra doesn't exist yet, gate this verification on a
  future "visual harness" task — for v2, hand-verify via `/screenshot`.

### Verification
- `/screenshot` — outer cyan halo visible around shell rounded edge; thin top
  highlight visible on each panel.

## Phase 3 — per-panel halo / highlight tuning

### Critical files
- `examples/aichat/src/main.rs`

### Changes
At each existing `GlassPanel` declaration, add:

| Site | line | `halo_strength` | `halo_radius` | `highlight_strength` |
|---|---|---|---|---|
| `app_shell` | 480 | **0.45** | 14.0 | 0.0 (shell is frame, no top band) |
| `sidebar` | 500 | 0.0 | (n/a) | **0.18** |
| `main_area` | 685 | 0.0 | (n/a) | **0.14** |
| `composer` | 828 | **0.25** | 10.0 | **0.22** |

Remove `use_scene_blur: 1.0` and `blur_amount: ...` lines from each site (the
shader keeps them as no-op for one transition commit, but the call sites can
drop them now). Also remove `specular_strength` since it's deprecated.

### Verification
- `/screenshot` — visual confirmation matches the 4/27 reference: shell glows
  cyan outside its edge, sidebar/main/composer have visible top brightness.

## Phase 4 — slider behavior + acceptance test wiring

### Critical files
- `examples/aichat/src/main.rs`

### Changes
- Update slider min/max/default to match v2 spec (slider is `[0%, 100%]`,
  default 90%). The slider widget currently maps the slider value 1:1 to the
  alpha — that's fine since `glass_opacity_values` interprets the slider value
  as "slider position" not "alpha".
- Add new acceptance test scaffolding for the v2 scenarios in
  `aichat-liquid-glass-v2.spec.md`. These tests don't need a running window —
  they parse the script_mod source string for `window.transparent: true`,
  `pass.clear_color: #00000000`, etc. Pixel-sampling tests are deferred to
  whenever Makepad gets headless render harness; document this gap in the
  spec scenario notes (already done — spec says "captured and sampled").

### Verification
- `cargo test -p makepad-example-aichat` passes.
- Manual sweep: drag slider 10% → 100%, observe smooth ghost-to-opaque
  transition with desktop visible at low end, near-opaque at 100%.

## Cross-phase verification

End-to-end visual check vs reference:

1. Run `cargo run -p makepad-example-aichat` over a desktop with a wallpaper
   that has clear regional color variation (the spec example uses mountain
   landscape).
2. `/screenshot` and compare to the 4/27 reference image.
3. Confirm:
   - Mountain visible behind sidebar, main area, and around shell edges.
   - Cyan glow ring extending outside the shell rounded corner.
   - Each panel has a thin bright top edge.
   - Sidebar `新对话` active capsule still looks correct (cyan capsule + ✦).
   - Backend selector + Glass slider + percentage label still aligned.
   - Composer暖金 send button still warm-gold against the panel.

If any of those fail, file a follow-up issue under
`examples/aichat/issues/` and link it from this plan.

## What v2 does **not** do

- No real backdrop blur. Through-glass transmission is sharp, not frosted.
  Real blur is queued for a v3 plan: NSVisualEffectView injection on macOS,
  Pass-based separable Gaussian elsewhere.
- No iridescent / animated halo. Static glass.
- No light-mode glass theme.
- No new `Pass` infrastructure. Everything ships within
  `widgets/src/glass_panel.rs` instance parameters and `main.rs` values.

## Rollback

All changes are local to two files (`widgets/src/glass_panel.rs` +
`examples/aichat/src/main.rs`). Revert these two files to restore v1 visual.
The deprecated `use_scene_blur` / `blur_amount` instance params are kept as
no-ops in this commit, so any other consumer of `GlassPanel` (none exist
in-tree at present) won't break.
