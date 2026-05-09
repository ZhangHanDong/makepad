# Step 133: Programmatic Geometry Snapshot

Date: 2026-05-09

## Result

Studio release build `[150]` ran:

```text
makepad-example-aichat-macos-native-clear-geometry-probe
```

The run reached State 4, installed one native glass container with four panels,
then executed the self-driven resize/reposition probe.

## Root Cause

The previous programmatic geometry probe emitted `WindowGeomChange` by calling
`MacosApp::do_callback` from inside `handle_platform_ops`.

That path is reentrant: `handle_platform_ops` runs while the current macOS
callback is already active, so the global callback has been temporarily taken.
The nested `do_callback` therefore had no callback to invoke and the synthetic
`WindowGeomChange` was dropped.

## Fix

The macOS event loop now shares one internal `WindowGeomChange` handling path.
Native macOS `WindowGeomChange` events and programmatic `ResizeWindow` /
`RepositionWindow` ops both route through that path directly.

When `MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT` is enabled, the programmatic op
path also logs whether the `NSWindow` geometry changed synchronously.

## Runtime Evidence

Build `[150]` logged:

```text
[liquid-glass] native-geometry-probe=request-resize position=(180.0,120.0) size=(980.0,760.0)
[liquid-glass] native-geometry-op=resize old_pos=(1270.0,532.0) new_pos=(1270.0,532.0) old_size=(900.0,700.0) new_size=(980.0,760.0) requested=(980.0,760.0) changed=true
[liquid-glass] native-display-frame-snapshot reason=geometry-change old_dpi=1.000 new_dpi=1.000 old_pos=(1270.0,532.0) new_pos=(1270.0,532.0) containers=1
[liquid-glass] native-geometry-op=reposition old_pos=(1270.0,532.0) new_pos=(180.0,120.0) old_size=(980.0,760.0) new_size=(980.0,760.0) requested=(180.0,120.0) changed=true
[liquid-glass] native-display-frame-snapshot reason=geometry-change old_dpi=1.000 new_dpi=1.000 old_pos=(1270.0,532.0) new_pos=(180.0,120.0) containers=1
```

Both snapshots were followed by four `native-panel-frame` lines, one for each
installed panel.

## Meaning

Programmatic same-screen resize/reposition now proves that native glass frame
snapshot logging follows actual `NSWindow` geometry changes.

This closes the self-driven geometry probe gap. It still does not prove real
Stage Manager, split-view, fullscreen, or multi-display behavior.
