# Step 134: Fullscreen Probe Runtime

Date: 2026-05-09

## Result

Studio release build `[155]` ran:

```text
makepad-example-aichat-macos-native-clear-fullscreen-probe
```

The run reached State 4, entered fullscreen, switched away from native underlay
while fullscreen, requested exit, observed exit, and restored the Apple native
underlay afterward.

## Root Cause

Two platform details blocked the earlier fullscreen probe:

- The aichat borderless standard window had `collectionBehavior=0`, so
  `toggleFullScreen:` did not enter fullscreen on the Step 128/151 runs.
- After adding fullscreen-primary behavior, the fullscreen delegate still
  delivered geometry changes while `handle_platform_ops` was already inside the
  active macOS callback. Those reentrant `send_change_event()` calls could not
  reach the app-level event handler reliably.

## Fix

Standard macOS windows now include
`NSWindowCollectionBehaviorFullScreenPrimary`. Fullscreen auxiliary windows keep
`NSWindowCollectionBehaviorFullScreenAuxiliary` and do not set primary.

Programmatic fullscreen and normalize ops now generate explicit old/new
`WindowGeomChange` events through the same internal path used by native macOS
geometry changes.

## Runtime Evidence

Build `[155]` logged:

```text
[liquid-glass] native-fullscreen-op=enter old_fullscreen=false new_fullscreen=true old_size=(900.0,700.0) new_size=(3440.0,1440.0) style_mask=32776 collection_behavior=128
[liquid-glass] fullscreen-native-fallback=shader reason=fullscreen-enter
[liquid-glass] native-fullscreen-probe=observed-enter
[liquid-glass] native-fullscreen-probe=request-exit
[liquid-glass] native-fullscreen-op=exit old_fullscreen=true new_fullscreen=true old_size=(3440.0,1440.0) new_size=(3440.0,1440.0) style_mask=49160 collection_behavior=128
[liquid-glass] fullscreen-native-restore=apple-native-underlay reason=fullscreen-exit
[liquid-glass] native-fullscreen-probe=observed-exit
```

The post-exit restore was followed by native batch reinstall evidence:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

No `native-fullscreen-probe=timeout phase=enter` was logged after the fix.

## Meaning

The fullscreen probe now validates the v4.1 policy: Apple native underlay is
suppressed while fullscreen and restored after exit.

This does not mean full native glass is supported inside fullscreen. Native
fullscreen glass remains deliberately out of scope for this phase.
