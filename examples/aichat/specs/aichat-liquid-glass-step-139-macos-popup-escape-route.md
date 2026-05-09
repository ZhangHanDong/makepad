# Step 139 - macOS Popup Escape Dismiss Route

Date: 2026-05-10

## Change

macOS popup windows now translate AppKit `keyDown:` Escape events into the
existing Makepad popup dismissal event:

```text
Event::PopupDismissed(PopupDismissedEvent {
    reason: PopupDismissReason::Escape,
})
```

The route is limited to `MacosWindow::is_popup`; normal windows still keep the
existing key handling behavior.

The platform logs the route when it fires:

```text
[liquid-glass] transient-window=popup-dismiss event=escape popup=...
```

## Studio Run

RunItem:

```text
makepad-example-aichat-macos-native-clear-transient-probe
```

Studio release build:

```text
[169]
```

## Evidence

Build `[169]` reached the existing transient popup native glass gates:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
```

Automation attempts used:

```text
osascript -e 'tell application "System Events" to key code 53'
osascript -e 'tell application "System Events" to set frontmost of process "makepad-example-aichat" to true' -e 'delay 0.2' -e 'tell application "System Events" to key code 53'
/opt/homebrew/bin/cliclick kp:esc
```

These attempts produced no new `transient-window=popup-dismiss event=escape`
or `transient-window-probe=dismissed ... reason=Escape` logs. This matches the
earlier limitation seen with automated system-click probes: the automation
environment does not currently prove real AppKit input delivery to the
Studio-launched child app.

## Verdict

The macOS AppKit Escape route is implemented in code and compile-verified, but
the physical/system Escape validation gate remains open until a manual or
trusted system input run produces both:

```text
[liquid-glass] transient-window=popup-dismiss event=escape ...
[liquid-glass] transient-window-probe=dismissed ... reason=Escape
```
