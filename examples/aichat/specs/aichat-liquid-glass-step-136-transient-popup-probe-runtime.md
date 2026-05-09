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

Studio release build:

```text
[160]
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

The main native batch stayed installed after the popup probe:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] container=0000000000000010 state=Installed reason=installed panels_installed=4 panels_failed=0
```

## Verdict

This closes the first macOS transient-window substrate installation gate:
`CxOsOp::CreatePopupWindow` can install an independent native glass substrate
for the popup window without reusing the main window's native container.

It does not close the full Phase I popup/modal gate. Still missing:

- visual proof of popup glass composition with popup content,
- popup widget-tree descriptor collection,
- `PopupDismissed` runtime delivery evidence,
- UIKit transient-window support or stable unsupported logging,
- modal-window support.
