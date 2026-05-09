# Step 129: Spacing Animation Runtime

Date: 2026-05-09

## Result

Studio release build `[140]` ran:

```text
makepad-example-aichat-macos-native-clear-spacing-probe
```

The probe started and drove the native container spacing animation:

```text
[liquid-glass] native-spacing-animation-probe=start
[liquid-glass] native-spacing-animation-probe frame=0 spacing=12.000
[liquid-glass] native-spacing-animation-probe frame=15 spacing=18.000
[liquid-glass] native-spacing-animation-probe frame=105 spacing=18.000
[liquid-glass] native-spacing-animation-probe frame=120 spacing=12.000
[liquid-glass] native-spacing-animation-probe=stop frame=120
```

During the run, the macOS backend repeatedly received updated native container
spacing and kept the native batch installed:

```text
[liquid-glass] native-container-spacing container=0000000000000010 spacing=12.400
[liquid-glass] native-container-spacing container=0000000000000010 spacing=18.000
[liquid-glass] native-container-spacing container=0000000000000010 spacing=19.600
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] container=0000000000000010 state=Installed reason=installed panels_installed=4 panels_failed=0
```

## Meaning

This proves the automatic spacing animation probe path can repeatedly invalidate
and reinstall/update the macOS native batch without falling out of State 4.

It does not prove the full Apple morphing visual quality. That still needs a
manual visual verdict focused on panel fusion/morph behavior and a resize pass
after animation to rule out stale panel frames.
