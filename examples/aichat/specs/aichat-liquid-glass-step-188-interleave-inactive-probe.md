# Step 188 - Interleave Inactive Probe

Date: 2026-05-13

## Context

Step 187 repeated geometry and spacing probes for the macOS
`AppleNativeInterleave` experimental backend. The inactive-window probe still
needed interleave-specific runtime evidence.

## Verification

Studio release build:

```text
build_id=[47]
runnable=makepad-example-aichat-apple-native-interleave-inactive-probe
```

The run used:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INACTIVE_PROBE=1
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=production
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

Startup/native install evidence:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

Initial inactive probe log:

```text
[liquid-glass] native-inactive-probe active=true style=clear multiplier=1.000
```

The first `Finder` activation attempt did not emit an inactive event because
the aichat app had not first become the frontmost app. The passing sequence was:

```text
osascript -e 'tell application "System Events" to set frontmost of process "makepad-example-aichat" to true'
osascript -e 'tell application "Finder" to activate'
```

Result:

```text
[liquid-glass] native-inactive-probe active=true style=clear multiplier=1.000
[liquid-glass] native-inactive-probe active=false style=clear multiplier=0.880
```

## Verdict

`AppleNativeInterleave` now has runtime evidence for app active-to-inactive
foreground multiplier updates on macOS.

This does not finish inactive-window support. Bright and dark wallpaper visual
assessment is still required to prove foreground readability and to ensure the
system native material behavior plus Makepad foreground multiplier do not
double-dim the UI.
