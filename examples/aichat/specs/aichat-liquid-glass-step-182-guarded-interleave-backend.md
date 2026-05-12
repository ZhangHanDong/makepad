# Step 182 - Guarded AppleNativeInterleave Backend

Date: 2026-05-12

## Context

Before this step, `AICHAT_GLASS_BACKEND=apple-native-interleave` was reserved
but always fell back to `ShaderBackdropInterior`. The production interleave work
was only reachable through `macos-native-clear` probe runnables.

This step keeps the default fallback, but adds an explicit guarded opt-in:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_ENABLE_APPLE_NATIVE_INTERLEAVE=production-preview
```

## Implementation

The guarded opt-in resolves startup glass to native clear glass so descriptor
export can install the native batch immediately. Without the guard, the
reserved backend still falls back with the existing Step 98 warning.

New Studio runnable:

```text
makepad-example-aichat-apple-native-interleave-production-preview
```

The runnable sets:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_ENABLE_APPLE_NATIVE_INTERLEAVE=production-preview
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

## Verification

Commands:

```text
cargo test -p makepad-example-aichat --release aichat_apple_native_interleave
cargo check -p makepad-example-aichat --release
```

Result:

```text
4 passed; 0 failed
cargo check finished release profile
```

Studio release build:

```text
build_id=[36]
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave guarded production preview enabled; manual visual validation still required
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer frame=(3.0,3.0,894.0,694.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

Studio screenshot path:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-36-kind-0-req-3185-1778598457312.png
```

Foreground alpha histogram:

```text
size=(900, 700)
zero_alpha=589869
nonzero_alpha=40131
opaque_alpha=1706
partial_alpha=38425
```

## Verdict

`AppleNativeInterleave` is now a guarded production-preview backend, not only a
name reserved in the parser. It is still not the default or a completed
production route because the preview requires a manual system-composited visual
verdict.
