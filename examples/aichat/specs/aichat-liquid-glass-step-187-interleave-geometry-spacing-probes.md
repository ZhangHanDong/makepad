# Step 187 - Interleave Geometry and Spacing Probes

Date: 2026-05-13

## Context

Step 185 made macOS `AppleNativeInterleave` an explicit experimental backend.
Step 186 proved its fullscreen fallback/restore gate. The remaining automated
macOS advanced-behavior probes still needed to be repeated against the
interleave backend rather than only the older `macos-native-clear` underlay
runnables.

## Geometry Probe

Studio release build:

```text
build_id=[44]
runnable=makepad-example-aichat-apple-native-interleave-geometry-probe
```

The run used:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT=1
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
[liquid-glass] native-geometry-probe=schedule-resize
[liquid-glass] native-geometry-probe=request-resize position=(180.0,120.0) size=(980.0,760.0)
[liquid-glass] native-geometry-op=resize ... old_size=(900.0,700.0) new_size=(980.0,760.0) ... changed=true
[liquid-glass] native-display-frame-snapshot reason=geometry-change ... containers=1
[liquid-glass] native-panel-frame ... panel=0000000000000011 ... visible=true
[liquid-glass] native-panel-frame ... panel=0000000000000012 ... visible=true
[liquid-glass] native-panel-frame ... panel=0000000000000023 ... visible=true
[liquid-glass] native-panel-frame ... panel=0000000000000039 ... visible=true
[liquid-glass] native-geometry-op=reposition ... new_pos=(180.0,120.0) ... changed=true
[liquid-glass] native-display-frame-snapshot reason=geometry-change ... containers=1
```

Verdict: the interleave backend keeps native panel frame snapshots available
through self-driven resize and reposition operations. This proves same-display
logical-coordinate recomputation for the probe path; it does not prove real
multi-display movement or Stage Manager behavior.

## Spacing Probe

Studio release build:

```text
build_id=[45]
runnable=makepad-example-aichat-apple-native-interleave-spacing-probe
```

The run used:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_SPACING_PROBE=animate
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

Key logs:

```text
[liquid-glass] native-spacing-animation-probe=start
[liquid-glass] native-spacing-animation-probe frame=0 spacing=12.000
[liquid-glass] native-container-spacing container=0000000000000010 spacing=12.400
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
[liquid-glass] native-container-spacing container=0000000000000010 spacing=14.000
[liquid-glass] native-spacing-animation-probe frame=120 spacing=12.000
[liquid-glass] native-spacing-animation-probe=stop frame=120
```

Verdict: animated `GlassContainer.spacing` updates keep the interleave native
batch installed through the probe sequence. This proves the update path and
install stability, not final visual morph quality.

## Overall Verdict

`AppleNativeInterleave` now has automated macOS probe coverage for:

- fullscreen fallback/restore
- same-display geometry resize/reposition snapshots
- animated spacing update stability

The overall Apple native Liquid Glass objective remains incomplete because iOS
26 runtime validation, real multi-display/Stage Manager validation, inactive
bright/dark wallpaper visual assessment, popup/modal completion, runtime
switching, and final production-default criteria remain open.
