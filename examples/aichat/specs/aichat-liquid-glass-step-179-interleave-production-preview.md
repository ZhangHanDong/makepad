# Step 179 - Interleave Production Preview

Date: 2026-05-12

## Context

Step 178 proved the native interleave layer stack with the diagnostic lower
scene. This step adds a production-oriented preview target that keeps the same
layer stack but removes the visible diagnostic grid:

```text
makepad-example-aichat-macos-native-clear-interleave-production-preview
```

The target sets:

```text
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_COMPOSITING_PROOF=transparent-overlay
```

## Runtime Evidence

Studio release build:

```text
build_id=[32]
```

Key logs:

```text
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] compositing-proof=transparent-overlay
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer frame=(3.0,3.0,894.0,694.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

Studio screenshot path:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-32-kind-0-req-2929-1778596723841.png
```

Screenshot alpha histogram:

```text
size=(900, 700)
zero_alpha=589869
nonzero_alpha=40131
opaque_alpha=1706
partial_alpha=38425
```

The Studio screenshot still cannot capture native AppKit material itself, but
the alpha histogram proves the upper Makepad foreground remains mostly
transparent. That keeps the native glass container visible to the system
compositor rather than being covered by an opaque primary Metal pass.

## Verdict

The production preview target is valid as a guarded native-interleave preview:

- it keeps the Step 178 proven layer order
- it installs native clear glass in State 4
- it removes the diagnostic grid with `grid_strength=0.000`
- it keeps foreground text, controls, widget queries, and Studio screenshots on
  the primary Makepad surface

This still does not promote `AppleNativeInterleave` to the default product
backend. A manual system-composited visual pass is still required to judge
whether the production lower scene has enough recognizable native glass
material, refraction, and liquid distortion after removing the diagnostic grid.
