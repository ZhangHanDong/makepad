# Step 135: Inactive Event Source

Date: 2026-05-09

## Result

The macOS platform now listens to app-level activation changes in addition to
window key changes:

- `applicationDidBecomeActive:` maps to `WindowGotFocus` for existing windows.
- `applicationDidResignActive:` maps to `WindowLostFocus` for existing windows.

This gives native inactive styling a platform event source even when
`windowDidResignKey:` is not sufficient.

## Runtime Attempt

Studio release build `[157]` ran:

```text
makepad-example-aichat-macos-native-clear-inactive-probe
```

The run reached State 4 and logged:

```text
[liquid-glass] native-inactive-probe active=true style=clear multiplier=1.000
```

Attempts to activate Finder with AppleScript did not make Finder frontmost in
the current automation environment. `System Events` continued to report the
host process as frontmost, so no app resign-active event was produced and no
`active=false` runtime evidence was observed.

## Meaning

The missing platform event source has been added, but inactive transition
validation remains open until it can be run in an environment that can actually
move focus away from the Studio-launched child app.
