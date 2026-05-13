# Step 185 - macOS Interleave Experimental Backend

Date: 2026-05-13

## Context

Step 184 recorded the first system-composited visual pass for the guarded
`AppleNativeInterleave` production preview. That unblocked promotion from a
guarded preview to an explicit macOS experimental backend.

## Implementation

`AICHAT_GLASS_BACKEND=apple-native-interleave` now directly selects the macOS
experimental native interleave route. It no longer requires
`AICHAT_ENABLE_APPLE_NATIVE_INTERLEAVE=production-preview`.

New Studio runnable:

```text
makepad-example-aichat-apple-native-interleave-experimental
```

The runnable sets:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

The route remains opt-in because users must explicitly request the
`apple-native-interleave` backend. It is not the default backend and it does not
claim iOS or advanced behavior completion.

## Verification

Commands:

```text
cargo test -p makepad-example-aichat --release aichat_apple_native_interleave
cargo check -p makepad-example-aichat --release
```

Result:

```text
3 passed; 0 failed
cargo check finished release profile
```

Studio release build:

```text
build_id=[39]
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

Studio screenshot path:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-39-kind-0-req-3345-1778667888187.png
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

macOS `AppleNativeInterleave` is now an explicit experimental backend.

The overall Apple native Liquid Glass objective remains incomplete because:

- iOS 26 UIKit runtime validation is still blocked locally
- fullscreen, Stage Manager, multi-display, popup/modal, and runtime switching
  remain limited or separate phases
- platform/widget logs still use the shared `apple-native-underlay` installer
  label where they are describing the native panel batch implementation
