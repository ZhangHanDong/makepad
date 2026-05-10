# Step 165 - macOS Native Control System Click Gate

Date: 2026-05-10

## Context

Step 164 proved that the post-geometry native-control mouse probe can re-run
when a control moves. The remaining macOS native-control gate was a trusted
system click: the visible `Clear` control had to be hit by AppKit as a native
`NativeGlassButton`, then bridge back into Makepad's existing button action
path.

## Runtime Setup

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-control-probe
build_id=[18]
commit=8d00587d
```

The app reached the native clear substrate and native-control install gates:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
```

The visible `Clear` button frame was:

```text
[liquid-glass] native-control-frame ... label="Clear" makepad=(828.0,564.0,78.0,36.0) appkit=(828.0,100.0,78.0,36.0) z_order=0 visible=true
```

System Events reported the window bounds:

```text
makepad-example-aichat window=(1270,208,900,700)
```

The first click attempt at the calculated screen point produced no app logs
because the app was not frontmost. After setting the process frontmost, a
`cliclick` system click at the same point entered the AppKit window.

## Passing Evidence

The successful click produced the full diagnostic chain:

```text
[liquid-glass] backend=apple-native-controls event=window-send-event type=NSLeftMouseDown point=(867.0,118.0) hit=NativeGlassButton
[liquid-glass] backend=apple-native-controls event=button-mouse-down window_point=(867.0,118.0)
[liquid-glass] backend=apple-native-controls event=target-action window_index=0 window_generation=0 control_id=67
[liquid-glass] backend=apple-native-controls event=button-action control_id=67
[liquid-glass] native-control-probe=makepad-click id=clear_button
```

This closes the macOS physical/system click gate for the native `Clear` button:
AppKit hit testing selected `NativeGlassButton`, the native target/action bridge
fired, and Makepad received the existing button action.

## Remaining Scope

This proves the macOS native-control click path for the aichat probe. It does
not validate UIKit runtime controls, keyboard/focus accessibility traversal,
VoiceOver behavior, or arbitrary app layouts.
