# Step 128: Fullscreen Probe Runtime

Date: 2026-05-09

## Result

Studio release build `[139]` ran:

```text
makepad-example-aichat-macos-native-clear-fullscreen-probe
```

The app entered the macOS native clear path and installed the native batch:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The fullscreen probe requested fullscreen:

```text
[liquid-glass] native-fullscreen-probe=request-enter
```

The probe did not observe a fullscreen-enter geometry event before its timeout:

```text
[liquid-glass] native-fullscreen-probe=timeout phase=enter
```

No later fullscreen fallback, restore, or exit evidence appeared in the Studio
log stream.

## Meaning

This run does not prove fullscreen native glass support. It confirms that the
fullscreen probe path is reachable, but the Studio-launched app did not produce
the fullscreen-enter geometry signal needed to validate native panel alignment,
explicit fallback, or restore behavior.

Fullscreen remains a Phase H open gate. A future run needs either a reliable
fullscreen-enter event in Studio or a manual/system-level validation path that
captures enter, fallback/restore, exit, and native panel frame evidence.
