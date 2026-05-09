# Step 131: Geometry Probe Runtime

Date: 2026-05-09

## Result

Studio release build `[142]` ran:

```text
makepad-example-aichat-macos-native-clear-geometry-probe
```

The app entered the macOS native clear path and installed the native batch:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The attempted geometry-change driver used System Events:

```applescript
tell application "System Events"
  tell process "makepad-example-aichat"
    set position of window 1 to {180, 120}
    set size of window 1 to {980, 760}
  end tell
end tell
```

System Events could see the process but reported no scriptable window:

```text
no-window
```

No `native-display-frame-snapshot reason=geometry-change` or `native-panel-frame`
logs appeared.

## Meaning

This run proves the geometry probe runnable reaches State 4, but it does not
prove Stage Manager, split-view, or resize/reposition behavior. The current
automation path cannot resize the Studio-launched borderless Makepad window via
System Events.

The geometry gate remains open. A future validation needs a reliable resize or
move driver, or a manual run that records `native-display-frame-snapshot` and
`native-panel-frame` logs after window geometry changes.
