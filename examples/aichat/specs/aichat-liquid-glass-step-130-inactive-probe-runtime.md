# Step 130: Inactive Probe Runtime

Date: 2026-05-09

## Result

Studio release build `[141]` ran:

```text
makepad-example-aichat-macos-native-clear-inactive-probe
```

The app entered the macOS native clear path and logged the active native
inactive-probe state:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] native-inactive-probe active=true style=clear multiplier=1.000
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

After activating Finder with:

```bash
osascript -e 'tell application "Finder" to activate'
```

no `native-inactive-probe active=false` log appeared in the Studio log stream.

## Meaning

This run proves the inactive probe path logs the native active state during a
native clear run. It does not prove inactive-window native glass behavior,
because the Studio-launched app did not emit the inactive transition evidence.

Inactive-window support remains a Phase H open gate. A future validation needs
active and inactive visual evidence on bright/dark wallpapers, plus explicit
`active=false` logs showing the intended native inactive multiplier.
