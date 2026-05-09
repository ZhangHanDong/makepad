# Step 137 - Transient Popup Widget Descriptor Export

Date: 2026-05-10

## Change

The macOS transient popup probe now draws a real `GlassContainer` widget tree
inside the popup pass instead of drawing only manual marker quads.

The popup tree contains one native `GlassPanel`. Because the popup pass is
owned by the popup `WindowHandle`, `GlassContainer` resolves the current window
id to the popup window and emits `CxOsOp::SetNativeGlassBatch` for that
transient window.

## Studio Run

RunItem:

```text
makepad-example-aichat-macos-native-clear-transient-probe
```

Studio release build:

```text
[168]
```

## Evidence

The main window reached native clear State 4:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

aichat opened the real popup:

```text
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
```

The platform installed the initial popup native substrate:

```text
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
```

The popup widget tree drew in the popup pass:

```text
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] transient-window-probe=draw popup_size=(240.0,160.0)
```

The main window kept its four native panels:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The popup window installed the widget-exported native batch with one panel:

```text
[liquid-glass] native-container-spacing container=0000000000000047 spacing=10.000
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
[liquid-glass] container=0000000000000047 state=Installed reason=installed panels_installed=1 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

Studio captured the popup framebuffer at:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-168-kind-0-req-98-1778344085662.png
```

## Verdict

This closes the macOS transient popup widget-descriptor gate for the probe
path. A popup window can now own a `GlassContainer` widget tree, export native
panel descriptors through the shared collector, and install the resulting
native batch on the popup window without stealing or replacing the main
window's native batch.

Still missing for full Phase I:

- physical outside-click or Escape dismissal evidence,
- UIKit transient-window support or stable unsupported logging,
- modal-window support.
