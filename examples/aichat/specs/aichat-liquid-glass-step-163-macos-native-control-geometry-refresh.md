# Step 163 - macOS Native Control Geometry Refresh

Date: 2026-05-10

## Context

After Step 162, the user performed a physical click against the visible aichat
window. The click entered the app window, but did not close the native-control
activation gate.

## Physical Click Evidence

Build `[4]` had already installed the native clear substrate and two AppKit
`NativeGlassButton` controls. After install log index `118`, physical mouse
activity produced `NSWindow.sendEvent:` and Metal-view diagnostics such as:

```text
[liquid-glass] backend=apple-native-controls event=window-send-event type=NSLeftMouseDown point=(313.0,146.0) hit=RenderViewClass
[liquid-glass] backend=apple-native-controls event=metal-view-mouse-down window_point=(313.0,146.0)
[liquid-glass] backend=apple-native-controls event=window-send-event type=NSLeftMouseUp point=(313.0,146.0) hit=RenderViewClass
```

The same query produced no native activation entries:

```text
target-action
button-action
native-control-probe=makepad-click
button-mouse-down
```

The event path therefore reached the window and Metal view, but not the native
button sibling.

## Root Cause

The native-control batch cache compares Makepad descriptor data. That is correct
for ordinary descriptor updates, but the macOS AppKit frame for a control is not
derived from descriptor data alone:

```text
appkit_y = container_view.bounds.height - makepad_y - makepad_height
```

After a window geometry change, the cached `NativeGlassControlBatch` can remain
descriptor-equivalent while the AppKit y-axis conversion must be recomputed using
the new `container_view.bounds.height`. If the platform layer skips the
descriptor-equivalent batch, the visible Makepad-rendered button can move with
layout while the AppKit `NativeGlassButton` sibling remains at the old AppKit
frame.

That matches the failed click evidence: `NSWindow.sendEvent:` saw the click, but
hit-testing resolved `RenderViewClass` instead of `NativeGlassButton`.

## Fix

macOS now refreshes installed native controls on window geometry changes:

- `MacosWindow::refresh_native_glass_control_frames_for_geometry_change()`
  clones the last installed native-control batch and forces a reinstall.
- The forced reinstall bypasses the descriptor-equivalent early return.
- The reinstall recomputes every AppKit control frame from current
  `container_view.bounds`.
- The refresh logs:

```text
[liquid-glass] backend=apple-native-controls event=frame-refresh reason=geometry-change controls_total=... controls_visible=...
```

This is intentionally scoped to native controls. The underlay path already has
its own native glass batch and geometry probes.

## Runtime Rerun

Studio remote release run after the fix:

```text
ClearBuild [4]
RunItem makepad-example-aichat-macos-native-clear-control-probe
build_id=[5]
```

Build `[5]` reached the install gates:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

## Verification Commands

```text
rustfmt --edition 2021 platform/src/os/apple/macos/macos_window.rs platform/src/os/apple/macos/macos.rs
cargo check -p makepad-platform --release
cargo test -p makepad-platform native_glass_window_geometry_changed_detects_position_size_and_dpi --release
cargo test -p makepad-platform native_glass_control --release
git diff --check
```

## Remaining Gate

This fix still needs a physical runtime rerun on build `[5]` or a later fresh
build:

1. Optionally resize the aichat window to trigger:

```text
[liquid-glass] backend=apple-native-controls event=frame-refresh reason=geometry-change ...
```

2. Click the visible `Clear` button and prove:

```text
window-send-event -> button-mouse-down -> target-action -> button-action -> native-control-probe=makepad-click
```

