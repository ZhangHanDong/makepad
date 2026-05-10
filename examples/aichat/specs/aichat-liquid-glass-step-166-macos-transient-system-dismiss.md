# Step 166 - macOS Transient System Dismiss Evidence

Date: 2026-05-10

## Context

Steps 139 and 140 implemented popup Escape and outside-click dismissal routes,
but previous automation did not prove trusted system input. After Step 165
confirmed that `cliclick` can deliver trusted mouse input to the
Studio-launched aichat process when the process is frontmost, the transient
popup gates were re-run.

## Escape Evidence

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-transient-probe
build_id=[19]
```

The run reached both main-window and popup-window native glass gates:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
```

After setting `makepad-example-aichat` frontmost and sending `cliclick kp:esc`,
the popup produced:

```text
[liquid-glass] transient-window=popup-dismiss event=escape popup=WindowId(1, 0)
[liquid-glass] transient-window-probe=dismissed popup=WindowId(1, 0) reason=Escape
```

This closes the macOS trusted system Escape dismissal gate for transient native
glass popups.

## Outside Click Evidence

A fresh transient run:

```text
RunItem makepad-example-aichat-macos-native-clear-transient-probe
build_id=[20]
```

again reached the popup native glass gates:

```text
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
```

System Events reported two aichat windows:

```text
popup=(1830,304,240,160)
main=(1270,208,900,700)
```

A trusted system click at `(1570,508)` was inside the main window and outside
the popup. The popup dismissed, but the reason was `FocusLost`:

```text
[liquid-glass] transient-window-probe=dismissed popup=WindowId(1, 0) reason=FocusLost
```

No `transient-window=popup-dismiss event=outside-click` log was produced.

This matches the current macOS event ordering: `MacosWindow::send_lost_focus_event`
maps popup focus loss to `PopupDismissReason::FocusLost` before a non-popup
`MouseDown` reaches `dispatch_popup_outside_click_if_needed`. Therefore trusted
physical/system outside dismissal is proved, but the specific `OutsideClick`
reason remains a Studio/shared-input route unless macOS popup focus-loss
semantics are intentionally changed later.

## Verdict

- macOS trusted system Escape dismissal: closed.
- macOS trusted system outside dismissal: closed as `FocusLost`.
- macOS trusted system outside dismissal with `OutsideClick` reason: not
  claimed; current AppKit ordering produces `FocusLost`.
