# Step 136 - macOS Transient Popup Native Glass Probe

Date: 2026-05-09

## Studio Run

RunItem:

```text
makepad-example-aichat-macos-native-clear-transient-probe
```

Injected environment:

```text
AICHAT_GLASS_BACKEND=macos-native-clear
MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE=1
```

Studio release builds:

```text
[160] initial native substrate install proof
[161] popup draw-pass proof
[166] synthetic PopupDismissed delivery proof
```

## Evidence

The main window reached native clear State 4:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

aichat then opened a real platform popup:

```text
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
```

macOS installed native glass on the popup window:

```text
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
```

The popup's own Metal pass drew content in build `[161]`:

```text
[liquid-glass] transient-window-probe=draw popup_size=(240.0,160.0)
```

The main native batch stayed installed after the popup probe:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] container=0000000000000010 state=Installed reason=installed panels_installed=4 panels_failed=0
```

The synthetic dismiss probe in build `[166]` dispatched
`Event::PopupDismissed` from the macOS platform layer and aichat received it:

```text
[liquid-glass] transient-window=popup-dismiss-probe request=dispatch-popup-dismissed
[liquid-glass] transient-window-probe=dismissed popup=WindowId(1, 0) reason=FocusLost
```

## Verdict

This closes the first macOS transient-window substrate installation gate:
`CxOsOp::CreatePopupWindow` can install an independent native glass substrate
for the popup window without reusing the main window's native container.

Build `[161]` also closes the empty-popup gap: the transient popup can own a
separate draw pass and present Makepad Metal content above its native glass
substrate. Studio captured the popup framebuffer at:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-161-kind-0-req-49-1778342316874.png
```

It does not close the full Phase I popup/modal gate. Still missing:

- popup widget-tree descriptor collection,
- physical outside-click or Escape dismissal evidence,
- UIKit transient-window support or stable unsupported logging,
- modal-window support.
