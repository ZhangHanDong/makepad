# Step 169 - Native Spacing Animation Rerun

Date: 2026-05-10

## Context

Step 129 proved that the native container spacing animation probe can run while
the native batch remains installed. This rerun records current evidence after
the later geometry/control/transient fixes.

## Runtime Evidence

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-spacing-probe
build_id=[22]
```

The run reached native clear State 4:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The spacing animation probe ran from 12 to 36 and back to 12:

```text
[liquid-glass] native-spacing-animation-probe=start
[liquid-glass] native-spacing-animation-probe frame=0 spacing=12.000
[liquid-glass] native-spacing-animation-probe frame=60 spacing=36.000
[liquid-glass] native-spacing-animation-probe frame=120 spacing=12.000
[liquid-glass] native-spacing-animation-probe=stop frame=120
```

During the run, the platform repeatedly logged native container spacing updates
and kept the native batch installed:

```text
[liquid-glass] native-container-spacing container=0000000000000010 spacing=12.400
[liquid-glass] native-container-spacing container=0000000000000010 spacing=14.800
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

## Verdict

The current macOS native backend accepts dynamic `GlassContainer.spacing`
updates and keeps all four native panels installed through the animation.

Remaining scope:

- manual visual assessment of Apple morph/merge quality
- resize-after-animation visual drift validation
