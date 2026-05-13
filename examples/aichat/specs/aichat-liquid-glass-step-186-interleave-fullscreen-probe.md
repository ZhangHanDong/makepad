# Step 186 - Interleave Fullscreen Probe

Date: 2026-05-13

## Context

Step 185 promoted macOS `AppleNativeInterleave` to an explicit experimental
backend. The earlier fullscreen evidence covered the `macos-native-clear`
underlay runnable, not the new interleave backend.

## Implementation

Added interleave-specific Studio runnables:

```text
makepad-example-aichat-apple-native-interleave-spacing-probe
makepad-example-aichat-apple-native-interleave-inactive-probe
makepad-example-aichat-apple-native-interleave-fullscreen-probe
makepad-example-aichat-apple-native-interleave-geometry-probe
```

Each runnable uses:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

The fullscreen probe initially exposed a race: build `[41]` requested
`disable_fullscreen` immediately after the first fullscreen-enter geometry event,
before the macOS fullscreen transition had settled. The platform log showed:

```text
native-fullscreen-op=exit old_fullscreen=true new_fullscreen=true
```

and a screenshot remained fullscreen-sized at `3440x1440`.

The probe now delays the exit request after observing fullscreen enter, then
waits for either an exit geometry event or an explicit exit timeout. This keeps
the probe aligned with AppKit's fullscreen transition timing instead of using
the first `willEnterFullscreen` geometry event as proof that exit can be
requested immediately.

## Verification

Commands:

```text
cargo test -p makepad-example-aichat --release aichat_native_fullscreen_probe
cargo fmt -p makepad-example-aichat --check
git diff --check
```

Result:

```text
4 passed; 0 failed
```

Studio release build:

```text
build_id=[42]
runnable=makepad-example-aichat-apple-native-interleave-fullscreen-probe
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
[liquid-glass] native-fullscreen-probe=request-enter
[liquid-glass] native-fullscreen-op=enter old_fullscreen=false new_fullscreen=true old_size=(900.0,700.0) new_size=(3440.0,1440.0)
[liquid-glass] fullscreen-native-fallback=shader reason=fullscreen-enter
[liquid-glass] native-fullscreen-probe=observed-enter
[liquid-glass] native-fullscreen-probe=request-exit
[liquid-glass] native-fullscreen-op=exit old_fullscreen=true new_fullscreen=false old_size=(3440.0,1440.0) new_size=(900.0,700.0)
[liquid-glass] fullscreen-native-restore=apple-native-underlay reason=fullscreen-exit
[liquid-glass] native-fullscreen-probe=observed-exit
```

Post-exit Studio screenshot:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-42-kind-0-req-3521-1778668948025.png
width=900
height=700
```

## Verdict

The macOS `AppleNativeInterleave` fullscreen fallback/restore probe now passes.

This is still not full native fullscreen Liquid Glass support. The current
policy remains explicit fallback to shader while fullscreen is active, then
restore native glass after fullscreen exit.
