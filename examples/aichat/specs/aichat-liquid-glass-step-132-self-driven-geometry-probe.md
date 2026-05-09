# Step 132: Self-Driven Geometry Probe

Date: 2026-05-09

## Result

The aichat geometry probe no longer depends only on external window automation.
When `MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT=1` is enabled and the native batch
reports `reason=installed-native-glass-batch`, aichat schedules a next-frame
self resize/reposition request:

```text
[liquid-glass] native-geometry-probe=schedule-resize
[liquid-glass] native-geometry-probe=request-resize position=(180.0,120.0) size=(980.0,760.0)
```

The macOS platform op handlers for `ResizeWindow` and `RepositionWindow` now
capture the old geometry before setting the `NSWindow` frame and emit a
`WindowGeomChange` using that explicit old geometry.

## Runtime Evidence

Studio release builds `[143]` and `[144]` showed that starting the geometry
probe too early, on `reason=installed-on-proofed-hierarchy`, could request
resize before the native batch was the active evidence target.

Studio release build `[146]` moved the request to the next frame after
`installed-native-glass-batch` and logged:

```text
[liquid-glass] native-geometry-probe=schedule-resize
[liquid-glass] native-geometry-probe=request-resize position=(180.0,120.0) size=(980.0,760.0)
```

Studio release build `[147]` used explicit old-geometry events for macOS
programmatic resize/reposition and logged the same schedule/request pair.

Neither build produced `native-display-frame-snapshot reason=geometry-change`
or `native-panel-frame` logs.

## Meaning

The geometry probe is now self-driven and reaches the resize/reposition request
path, but it still does not prove Stage Manager, split-view, or geometry-change
native frame behavior in a Studio run.

The remaining gap is lower in the event/pump path: the request is logged, but no
native frame snapshot is observed afterward. A future slice needs to determine
whether the platform ops are not being drained during this probe, whether the
macOS frame change is not being applied to this borderless Studio-launched
window, or whether the frame snapshot gate needs to observe a later paint/tick.
