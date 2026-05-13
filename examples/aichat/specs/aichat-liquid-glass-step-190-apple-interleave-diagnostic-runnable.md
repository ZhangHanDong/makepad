# Step 190 - Apple Interleave Diagnostic Runnable

Date: 2026-05-13

## Context

After the macOS `AppleNativeInterleave` backend was promoted to an explicit
experimental runnable, current manual feedback reported that the production
profile could still look opaque or visually too subtle. Studio framebuffer
screenshots cannot settle this because they capture only the foreground
Makepad surface and omit the AppKit `NSGlassEffectView` composition.

Step 190 adds a high-contrast visual diagnostic runnable on the same explicit
`apple-native-interleave` backend. It is not a production style. It exists to
separate two questions:

- whether the native interleave hierarchy is installed and sampling lower
  content
- whether the production profile is visually strong enough for users to
  perceive as Liquid Glass

## Implementation

New Studio runnable:

```text
makepad-example-aichat-apple-native-interleave-diagnostic-overlay
```

The runnable sets:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=diagnostic
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

Compared with `makepad-example-aichat-apple-native-interleave-experimental`,
only the lower-scene profile changes from `production` to `diagnostic`.

## Verification

Studio release build:

```text
build_id=[58]
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] compositing-proof=transparent-overlay
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=diagnostic proof=Refraction grid_strength=0.180 detail_strength=0.260
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

Studio screenshot remains a foreground-only diagnostic:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-58-kind-0-req-4353-1778671225175.png
```

It is expected to show the black/transparent foreground view, not the AppKit
native glass composition. Manual system-composited viewing is still required
for material quality.

## Verdict

The explicit macOS `AppleNativeInterleave` backend now has both:

- production preview: `makepad-example-aichat-apple-native-interleave-experimental`
- high-contrast diagnostic: `makepad-example-aichat-apple-native-interleave-diagnostic-overlay`

If the diagnostic overlay shows clear refraction/grid response while the
production preview looks flat, the next task is production visual tuning. If
the diagnostic overlay also looks opaque or black, the remaining blocker is in
native/macOS composition visibility rather than production style tuning.
