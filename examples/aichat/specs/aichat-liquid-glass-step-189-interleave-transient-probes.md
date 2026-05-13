# Step 189 - Interleave Transient Probes

Date: 2026-05-13

## Context

Phase I popup/transient native glass had macOS proof for the older
`macos-native-clear` underlay runnables. After Step 185 promoted macOS
`AppleNativeInterleave` to an explicit experimental backend, the transient
probe path needed to be repeated with the interleave main-window stack active.

## Implementation

Added Studio runnables:

```text
makepad-example-aichat-apple-native-interleave-transient-probe
makepad-example-aichat-apple-native-interleave-transient-dismiss-probe
```

Both use:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

The first also sets:

```text
MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE=1
```

The second sets:

```text
MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE=dismiss
```

## Popup Install Probe

Studio release build:

```text
build_id=[50]
runnable=makepad-example-aichat-apple-native-interleave-transient-probe
```

Main-window interleave evidence:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

Popup evidence:

```text
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
[liquid-glass] transient-window=popup state=Rejected reason=backend-not-native
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] transient-window-probe=draw popup_size=(240.0,160.0)
[liquid-glass] native-container-spacing container=0000000000000047 spacing=10.000
[liquid-glass] native-interleave-hierarchy reason=native-container-installed subviews=2
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

The early `backend-not-native` line is emitted before the popup-local widget
tree contributes its native glass batch. The popup-local batch then installs
successfully with one panel and its own native container, while the main window
keeps the interleave stack and four installed panels.

## Dismiss Probe

Studio release build:

```text
build_id=[51]
runnable=makepad-example-aichat-apple-native-interleave-transient-dismiss-probe
```

Key logs:

```text
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
[liquid-glass] transient-window=popup state=Rejected reason=backend-not-native
[liquid-glass] transient-window=popup-dismiss-probe request=parent-make-key
[liquid-glass] transient-window-probe=dismissed popup=WindowId(1, 0) reason=FocusLost
[liquid-glass] transient-window=popup-dismiss-probe request=dispatch-popup-dismissed
```

This dismiss probe intentionally closes before popup-local draw/native install
evidence appears. Popup-local native install is covered by build `[50]`; build
`[51]` covers the dismiss route while the interleave main-window backend is
active.

## Verdict

`AppleNativeInterleave` now has macOS transient-window probe coverage for:

- main-window interleave stack active while opening a popup
- popup-local widget-tree descriptor export
- popup-local native batch installation with one panel
- FocusLost dismiss delivery

Phase I remains incomplete because UIKit transient implementation/runtime
validation and separate-platform-window modal native glass remain unproven.
