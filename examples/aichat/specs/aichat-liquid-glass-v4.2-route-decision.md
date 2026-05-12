# aichat Liquid Glass v4.2 Route Decision

## Selected Route: ShaderBackdropInterior

v4.2 selects `ShaderBackdropInterior` for complete aichat interior Liquid
Glass. Apple native APIs remain useful for underlay/substrate diagnostics and
future platform affordances, but the current Makepad macOS renderer should not
claim full native Liquid Glass for aichat interiors.

## Evidence

### v4.1 Underlay

The v4.1 native underlay path installed native panels below the Makepad Metal
view and kept input owned by Makepad. It reached native State 4 and showed edge
or substrate effects, but the user could not identify meaningful full interior
glass. That path is `AppleNativeUnderlay`, not full native Liquid Glass.

The edge-heavy visual result is expected for this compositing model. The native
`NSGlassEffectView` / `NSGlassEffectContainerView` hierarchy is below the
transparent Makepad Metal view, while aichat's panels, cards, readability
overlays, input field, text, and generated UI are painted above it. The native
material is therefore most visible where Makepad coverage is thinnest: rounded
edges, rims, halos, gaps, and other low-alpha regions. The opaque or
semi-opaque interior layers hide most of the native material in the middle of a
panel.

This is an architecture boundary, not just a tuning issue. More aggressive
tint/noise/halo tuning can make the underlay easier or harder to perceive, but
it cannot make AppKit sample and refract the already-rendered Makepad interior
content through a single Metal layer. Full interior glass requires either a
Makepad-rendered backdrop/refraction route or a later native interleave renderer
with separate lower and upper Makepad surfaces.

### Above-Metal Probe

The v4.2 above-Metal probe installed a diagnostic `NSGlassEffectView` above the
Makepad Metal view:

```text
[liquid-glass] above-metal-probe state=installed style=clear style_raw=1 input=diagnostic-overlay
```

The Studio framebuffer screenshot showed only the Makepad pattern, while a
system screenshot showed the native rounded glass rectangle. This proves two
things:

- AppKit glass above Makepad Metal can be visible in real macOS composition.
- Studio framebuffer screenshot is not sufficient evidence for native AppKit
  overlays above the framebuffer.

The above-Metal hierarchy is still not a production aichat architecture because
the native view sits above Makepad-rendered text and controls.

### Two-Layer Interleave

AppleNativeInterleave is not selected for v4.2. A true interleave route requires
two Makepad-rendered surfaces or passes:

- lower scene/content surface for native glass to sample
- upper transparent foreground surface for text, controls, generated UI, and
  Studio-inspectable widgets

The current macOS path has one primary Makepad `CAMetalLayer`. Building a real
two-layer interleave renderer is a larger renderer split, not a small aichat
target change. A native-only upper label/control proof would not validate
Makepad foreground interleave, so v4.2 rejects that as a fake proof.

Step 96 later proved that a dedicated aichat pass can be rendered and routed to
the macOS lower scene surface. Step 98 recorded the manual visual verdict for
that prototype: the user saw only a gray/grid lower scene, with no transparency
and no recognizable blur/refraction/liquid distortion. That verdict keeps
`AppleNativeInterleave` as a prototype, not a production route.

Step 172 narrows the failure. The probe hierarchy can be installed as a lower
Makepad Metal sibling, then `NSGlassEffectContainerView`, then the primary
Makepad Metal view. The remaining native-interleave blocker is therefore the
Makepad render split: the primary surface still paints the full aichat UI and
interior readability layers above native glass instead of acting as a
transparent foreground-only surface.

Step 173 adds the next manual gate:
`makepad-example-aichat-macos-native-clear-interleave-transparent-overlay`.
That runnable combines the lower-scene pass, native clear glass, the primary
Metal surface, and transparent `GlassPanel` overlays. It is a diagnostic
foreground-split probe, not a production backend.

Step 174 fixes Studio screenshot routing for the interleave probes. Lower-scene
diagnostic drawables no longer consume default Studio screenshot requests before
the primary pass. This keeps Studio screenshots useful for the Makepad
foreground surface, while native AppKit material visibility still requires
manual/system visual validation.

Step 175 audits the fixed screenshot's alpha channel. The apparent black
background is mostly alpha=0, so the current transparent-overlay probe is close
to a foreground-only upper Makepad surface at the framebuffer level. The
remaining gate is manual/system validation of whether AppKit native glass is
visibly sampling the lower Makepad surface through that foreground.

Step 176 records that local `screencapture` automation returned a black frame,
so the current native-material visibility gate cannot be closed by Studio or by
the available system screenshot path. A user manual verdict or usable
system-composition screenshot is required before promoting or rejecting this
interleave probe visually.

Step 177 adds
`makepad-example-aichat-macos-native-clear-interleave-diagnostic-overlay`, which
uses the same interleave/transparent-overlay stack but switches the lower scene
to the stronger `Refraction` profile. Use this target for the next manual
native-material verdict.

Step 178 records that the diagnostic interleave visual gate passed with a
user-provided system screenshot. The lower diagnostic scene is visible broadly
through the native glass interior, not just at edges, while foreground UI remains
above the stack. The next route is no longer more proof of layer ordering; it is
turning the diagnostic split into a real aichat lower-scene / upper-foreground
implementation.

Step 179 adds
`makepad-example-aichat-macos-native-clear-interleave-production-preview`. It
uses the same proven interleave stack but selects a production lower-scene
profile with `grid_strength=0.000`, keeping the diagnostic grid out of the
preview while preserving State 4 native clear glass and a mostly transparent
primary foreground. This is still a guarded preview, not a default backend,
until a manual system-composited visual pass confirms that the non-diagnostic
lower scene produces enough recognizable native glass material.

Step 181 strengthens that production lower scene without reintroducing the
diagnostic grid. The production profile now logs `grid_strength=0.000` and
`detail_strength=0.220`, giving native glass more background detail to sample
while keeping the route guarded behind the production-preview runnable.

### Input Evidence

Input must remain Makepad-owned for aichat. The above-Metal glass view is a
diagnostic overlay only; it is not a native control host and must not become the
production hit-test owner. Native interactive controls and forwarding policies
belong to a later explicit platform phase.

## Backend Naming

Do not use "full native" for the current macOS target. User-facing and log
names should distinguish:

- `ShaderOverlay`: existing Makepad overlay material.
- `AppleNativeUnderlay`: v4.1 native substrate/panel proof below the Makepad
  Metal layer.
- `ShaderBackdropInterior`: future Makepad-rendered full interior blur,
  refraction, and readability treatment.
- `AppleNativeInterleave`: reserved for a later renderer split if Makepad gains
  separate lower and upper Metal surfaces.

Guarded backend value:

- `AICHAT_GLASS_BACKEND=apple-native-interleave` is a known but unsupported
  value. Step 99 keeps the name reserved and falls back to
  `ShaderBackdropInterior` with the warning
  `AppleNativeInterleave failed Step 98 visual verdict; falling back to
  ShaderBackdropInterior`. This prevents the reserved name from being mistaken
  for a working full-native backend while routing users to the current complete
  interior fallback.

## Next Implementation Slice

The next aichat implementation slice should start `ShaderBackdropInterior`:

1. Add an offscreen scene/backdrop capture pass.
2. Add blur and refraction sampling primitives.
3. Teach `GlassPanel` to sample the backdrop texture.
4. Keep Apple native underlay targets as diagnostics and platform comparison
   targets.

This route avoids overclaiming native Liquid Glass while still preserving the
Apple native proof work for future renderer experiments.
