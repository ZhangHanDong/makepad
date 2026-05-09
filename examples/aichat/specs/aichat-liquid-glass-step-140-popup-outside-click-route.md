# Step 140 - Popup Outside-Click Dismiss Route

Date: 2026-05-10

## Change

The shared Studio input path and macOS `MacosEvent::MouseDown` path now check
whether a mouse down in a non-popup window should dismiss the topmost popup.

When a popup is dismissed this way, the platform logs:

```text
[liquid-glass] transient-window=popup-dismiss event=outside-click popup=... source_window=...
```

and dispatches:

```text
PopupDismissed(reason=OutsideClick)
```

The app remains responsible for closing the popup window.

## Tests

Passed:

```text
cargo test -p makepad-platform popup_outside_click_helper --release
cargo check -p makepad-platform --release
cargo check -p makepad-example-aichat --release
```

The focused unit test verifies that clicks inside the popup window do not
request dismissal, while clicks in the main window select the created popup for
outside-click dismissal.

## Studio Run

RunItem:

```text
makepad-example-aichat-macos-native-clear-transient-probe
```

Studio release build:

```text
[171]
```

Build `[171]` first revalidated the existing transient native glass setup:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
```

Then a Studio remote click at `(300, 300)` outside the popup produced:

```text
[liquid-glass] transient-window=popup-dismiss event=outside-click popup=WindowId(1, 0) source_window=WindowId(0, 0)
[liquid-glass] transient-window-probe=dismissed popup=WindowId(1, 0) reason=OutsideClick
```

## Verdict

This closes the Studio/Makepad shared input outside-click gate for transient
popup probes. It does not by itself prove a physical AppKit outside-click from
the system event stream; that remains a manual/trusted-input validation item.
