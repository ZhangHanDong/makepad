# Step 171 - AppleNativeUnderlay Edge-Only Verdict

Date: 2026-05-10

## Context

The current aichat macOS native path reaches AppKit native glass State 4, but
manual visual validation reports that the liquid-glass effect is mainly visible
at panel edges.

## Cause

This is expected for `AppleNativeUnderlay`.

The installed AppKit hierarchy places `NSGlassEffectView` /
`NSGlassEffectContainerView` below the transparent Makepad Metal view. Makepad
then paints the aichat shell, panels, chat cards, input field, generated UI,
text, readability overlays, and shader treatments above that native layer.

The native material is therefore visible only where the upper Makepad content is
transparent enough to reveal it. In practice, the clearest reveal points are
rounded corners, rims, halos, gaps, and edge fades. The middle of the panel is
covered by Makepad-rendered surfaces, so the native material cannot visibly
refract or distort the already-rendered Makepad interior.

## Verdict

The edge-only appearance is an architectural limit of the current underlay
composition, not a simple NativeOverlay tuning bug.

Native visual tuning can still improve readability and reduce double
highlights, noise, and halo intensity. It cannot turn the current single-Metal
layer underlay into full interior AppKit Liquid Glass.

Full interior liquid-glass visuals need one of these later routes:

- `ShaderBackdropInterior`: Makepad captures a backdrop/offscreen scene and
  renders blur, refraction, distortion, and readability itself.
- `AppleNativeInterleave`: Makepad splits rendering into lower and upper
  surfaces so AppKit glass can sit between background content and foreground
  text/controls.

Until a native interleave renderer produces a new passing visual verdict,
`AppleNativeUnderlay` must not be described as complete native interior Liquid
Glass for aichat.
