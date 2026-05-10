# Step 173 - Native Interleave Transparent Overlay Probe

Date: 2026-05-10

## Context

Step 172 proved that the macOS probe hierarchy can be installed as:

```text
lower Makepad Metal sibling
native NSGlassEffectContainerView
primary Makepad Metal view
```

The next question is whether an existing transparent-overlay proof can be
combined with the lower-scene pass to create a minimal foreground-split visual
gate.

## Implementation

`makepad.splash` adds a dedicated Studio runnable:

```text
makepad-example-aichat-macos-native-clear-interleave-transparent-overlay
```

It combines:

- `AICHAT_GLASS_BACKEND=macos-native-clear`
- `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass`
- `AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay`

This keeps the native AppKit glass path active, draws the aichat lower-scene
probe into the lower Metal sibling, and zeros the Makepad `GlassPanel` overlay
alphas so the primary surface is closer to a foreground-only layer.

## Runtime Evidence

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-interleave-transparent-overlay
build_id=[26]
```

The run reached all intended gates:

```text
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] compositing-proof=transparent-overlay
[liquid-glass] native-lower-scene-pass=draw
[liquid-glass] native-interleave-layer-probe drawable=available
[liquid-glass] native-interleave-layer-probe lower-scene-role=draw
[liquid-glass] native-interleave-hierarchy reason=native-container-installed subviews=3
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer frame=(3.0,3.0,894.0,694.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

## Verdict

The combined probe is now available for manual visual validation of the next
native-interleave hypothesis:

- If the user sees broader interior native glass in this runnable, the next
  production slice is a real aichat foreground split.
- If the user still sees only edge effects, transparent `GlassPanel` overlays
  are insufficient and additional opaque aichat surfaces, cards, or pass clear
  behavior must be removed from the primary surface.

This step does not claim production `AppleNativeInterleave`; it creates the
smallest current runnable that exercises lower scene, native glass, upper
primary, and transparent panel overlay together.
