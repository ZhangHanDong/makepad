# Apple Native Liquid Glass Standard

## Status

Normative visual and engineering standard for the Apple native Liquid Glass
route.

Date: 2026-05-26

This document defines when Makepad/aichat may call an Apple-platform result
"native Liquid Glass." It converts the visual checklist into testable gates and
keeps the Apple native route separate from the Makepad `ShaderBackdrop` route.

## Sources

- Apple Liquid Glass technology overview: <https://developer.apple.com/documentation/technologyoverviews/liquid-glass>
- Apple Adopting Liquid Glass: <https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass>
- AppKit `NSGlassEffectView`: <https://developer.apple.com/documentation/appkit/nsglasseffectview>
- AppKit `NSGlassEffectContainerView`: <https://developer.apple.com/documentation/appkit/nsglasseffectcontainerview>
- UIKit `UIGlassEffect`: <https://developer.apple.com/documentation/uikit/uiglasseffect>
- UIKit `UIGlassContainerEffect`: <https://developer.apple.com/documentation/uikit/uiglasscontainereffect>
- Local API matrix: [APPLE-LIQUID-GLASS-API-MATRIX.md](APPLE-LIQUID-GLASS-API-MATRIX.md)
- Full native route spec: [aichat-liquid-glass-v4-apple-native-full.spec.md](aichat-liquid-glass-v4-apple-native-full.spec.md)
- Compositing decision: [aichat-liquid-glass-v4.2-route-decision.md](aichat-liquid-glass-v4.2-route-decision.md)

## Objective

The Apple native route must use Apple platform APIs as the source of material
truth. Makepad may provide geometry, lower-scene content, transparent foreground
content, readable text tokens, and fallback rendering. It must not fake native
Liquid Glass by adding shader stripes, diagnostic grids, static images, or
custom displacement effects and then calling that result "native."

The standard is:

```text
Apple APIs render the glass material.
Makepad supplies semantic surfaces and foreground content.
Manual/system-composited validation confirms visible native material.
Unsupported or incomplete paths are labeled honestly.
```

## Backend Boundaries

| Backend | Role | May claim native Liquid Glass? |
| --- | --- | --- |
| `AppleNativeUnderlay` | Native AppKit/UIKit panels below the Makepad Metal view. Useful for API install, geometry, style, tint, radius, and limited edge-visible material. | No, not by itself. It may claim "native API installed" or "native underlay visible." |
| `AppleNativeInterleave` | Lower Makepad scene, native Apple glass, upper transparent Makepad foreground. Candidate architecture for full macOS native visual material. | Yes, only after the visual gates below pass. |
| `ShaderBackdropInterior` | Makepad-rendered blur/refraction/displacement backend. Cross-platform visual route. | No. It may claim "Liquid Glass-like shader backdrop," not Apple native. |
| `ShaderOverlay` | Existing fallback overlay material. | No. |

## Required Apple API Semantics

### macOS

The native route must use:

- `NSGlassEffectContainerView` for grouping and spacing/morph proximity.
- `NSGlassEffectView` for each native glass panel.
- `NSGlassEffectView.Style.regular` and `.clear` via validated runtime raw
  values or typed SDK values when available.
- `tintColor` converted from Makepad sRGB colors.
- `cornerRadius`; capsule maps to `min(width, height) / 2`.
- `contentView` parenting for container/panel content where the runtime exposes
  it.

### iOS/iPadOS

The native route must use:

- `UIVisualEffectView` as the visual effect host.
- `UIGlassContainerEffect` for grouped panels and spacing/morph proximity.
- `UIGlassEffect` for panel material.
- `tintColor`, `style`, and `isInteractive` only according to the native-control
  ownership policy.

Until an iOS 26 runtime validates this path, iOS support remains preflight or
installer-skeleton status.

## Visual Acceptance Checklist

Each item is evaluated from a system-composited visual pass. Studio framebuffer
screenshots are useful for Makepad foreground alpha and geometry, but they do
not prove AppKit/UIKit native composition.

### 1. Dynamic Refraction

Pass:

- Background content visible through the glass changes non-uniformly across the
  interior when the window moves, resizes, or underlying content changes.
- The effect is visible beyond the rounded edge/rim area.
- The source is Apple native glass, not Makepad shader displacement.

Fail:

- Only the panel edge/radius shows material.
- The center is a flat gray, tinted, or opaque fill.
- A diagnostic grid or stripe pattern is the only visible "distortion."

### 2. Physical Blur

Pass:

- Background detail behind the native panel is softened by the system material.
- The blur remains separated from foreground text and controls.
- Foreground text stays crisp because it is rendered above the glass layer.

Fail:

- Foreground text is blurred together with the backdrop.
- Makepad paints an opaque or semi-opaque root over the native material.
- The panel reads as a plain translucent rectangle without system blur.

### 3. Edge Highlight and Environment Reflection

Pass:

- Native edge highlights/rim lighting are visible on bright and dark wallpapers.
- App-side `NativeOverlay` highlights, noise, and halo are reduced enough to
  avoid double highlights.
- The edge reacts plausibly to surrounding light and window content.

Fail:

- Makepad shader highlights dominate the native rim.
- The panel has a static border that looks unrelated to the environment.
- The highlight disappears on either bright or dark backgrounds.

### 4. Light Dispersion

Pass:

- Subtle chromatic or spectral edge behavior appears where the system material
  provides it, especially near high-contrast background detail.
- The effect remains subtle and does not reduce text readability.

Allowed limitation:

- If the same OS/runtime does not show visible chromatic dispersion in Apple
  standard controls, absence of this signal is recorded as a platform visual
  limitation rather than an implementation failure.

Fail:

- Makepad adds fake rainbow/chroma overlays in the Apple native route.
- Color fringing appears on foreground text.

### 5. Interaction and Fluid Motion

Pass:

- Native controls use platform-native glass appearance and platform-owned
  interaction when the native-control phase is enabled.
- Container spacing changes can visibly merge or separate adjacent native glass
  panels.
- Resize and window movement keep native panels aligned without lag or stale
  frames.

Fail:

- Panels drift from Makepad layout during resize.
- Native views intercept input outside explicit native-control descriptors.
- Motion is implemented only as Makepad shader wobble while the backend is
  labeled Apple native.

### 6. Environment Adaptation

Pass:

- Light and dark wallpapers keep glass material visible and text readable.
- Increased contrast, Reduce Transparency, and Reduce Motion are tested.
- In the Apple native route, system material adaptation remains system-owned;
  Makepad adjusts foreground, separator, and readability tokens.

Fail:

- Bright wallpaper makes text unreadable.
- Dark wallpaper hides the material entirely.
- The app forcibly replaces Apple native glass with a custom opaque material
  instead of respecting the native backend policy.

### 7. Layering and Z Order

Pass:

- Background/lower scene, native glass, and foreground text are distinct layers.
- Foreground Makepad content is transparent except for actual text, controls,
  icons, and required readability surfaces.
- `z_order` only sorts native panels within the native container unless a later
  interleave renderer explicitly defines more layers.

Fail:

- The Makepad root paints a solid background over native glass.
- Native panels cover foreground text or steal clicks.
- Studio screenshot evidence is treated as proof of native AppKit/UIKit overlay
  material.

## Claim Levels

Use these names in docs, logs, PRs, and release notes.

| Claim | Minimum evidence |
| --- | --- |
| Native API Installed | State 4 log, class/selector preflight passed, native batch installed. |
| Native Underlay Visible | State 4 plus manual/system visual evidence that native material is visible at edges or transparent regions. |
| Native Interior Candidate | Lower scene, Apple native glass, and upper transparent foreground are all present; interior material is visible beyond edges in a manual/system-composited pass. |
| Apple Native Liquid Glass | Native Interior Candidate plus native container morph, native controls policy, readability/accessibility checks, resize/focus behavior, and platform-specific validation for the claimed OS family. |

State 4 alone is never enough for the last two claim levels.

## Engineering Gates

Before a native route can be promoted:

1. It runs through Studio release `RunItem`, not raw `cargo run`.
2. It logs native style, raw style value, container count, installed panel count,
   failed panel count, and app-facing substrate name.
3. It keeps native panel count within the current budget of 12 panels per
   window unless a later spec revises the budget.
4. It preserves resize behavior, including borderless-window resize affordances.
5. It records a manual visual verdict for bright and dark backgrounds.
6. It records whether the evidence comes from user eyesight, a system-composited
   screenshot, or a Studio framebuffer screenshot.
7. It does not mix `AppleNative` and `ShaderBackdrop` in one window unless a
   later compositing spec explicitly allows that hybrid.

## Diagnostic Assets Policy

Diagnostic grids, colored stripes, moving patterns, and exaggerated lower-scene
profiles are allowed only to prove layer ordering, transparency, and sampling.
They must be removed or disabled for production-preview visual claims.

A diagnostic target may pass a layer-ordering gate while failing the native
visual standard.

## Current Branch Interpretation

- `AppleNativeUnderlay` reaching State 4 proves native API installation and
  geometry synchronization. It does not prove complete Liquid Glass if the
  material is only visible at edges.
- A clear showcase target is useful for manual inspection, but it must remain
  transparent, resizable, and free of fake shader/refraction proof patterns.
- `AppleNativeInterleave` is the macOS-native candidate path for broad interior
  native material because it gives AppKit lower content to sample while keeping
  Makepad foreground content above the glass.
- iOS remains incomplete until an iOS 26 runtime validates UIKit glass surfaces,
  controls, rotation, safe areas, keyboard, split view, and Stage Manager.

## Non-Negotiable Failure Conditions

Do not promote or release a route as Apple Native Liquid Glass if any of these
are true:

- The user sees only an opaque gray panel.
- The only visible effect is a Makepad diagnostic grid, stripe, or shader warp.
- Native material is visible only at the edge but the claim says full/interior
  Liquid Glass.
- Text readability depends on blurring foreground content into the glass.
- Input is captured by native glass panels outside explicit native-control
  descriptors.
- iOS is claimed without iOS 26 runtime validation.
