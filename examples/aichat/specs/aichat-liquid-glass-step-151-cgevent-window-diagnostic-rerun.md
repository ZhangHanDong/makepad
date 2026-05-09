# Step 151 - CGEvent Window Diagnostic Rerun

Date: 2026-05-10

## Context

Step 150 added `NSWindow.sendEvent:` diagnostics for mouse events while the
aichat native-control probe is enabled. This rerun uses the existing CGEvent
probe after those diagnostics landed, to determine whether the retained system
event probe enters the window at all.

## Runtime Run

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-control-cgevent-probe
build_id=[3]
```

The app installed the native underlay and native controls:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-underlay state=Installed reason=installed-on-proofed-hierarchy
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The AppKit-side hit-test probe still resolved both native button mirrors:

```text
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000043 label="Clear" point=(734.5,492.0) result_class=NativeGlassButton matches_control=true
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000044 label="↑" point=(803.5,488.0) result_class=NativeGlassButton matches_control=true
```

The CGEvent probe fired with the corrected Quartz coordinate conversion:

```text
[liquid-glass] backend=apple-native-controls event=cg-event-probe mode=Global control=0000000000000043 label="Clear" local=(734.5,492.0) screen=(2004.5,1024.0) cg=(2004.5,416.0)
```

No subsequent `window-send-event`, `button-mouse-down`, `target-action`,
`button-action`, or `native-control-probe=makepad-click` logs appeared after
the CGEvent probe.

## Studio Remote Click Note

In the same Studio session, a Studio remote `Click` at the current widget-dump
center of `clear_button` also produced no native/window event logs. This is not
treated as AppKit click evidence: Studio remote clicks are insufficient for the
native-control physical/system click gate unless paired with the native
diagnostic chain.

## Verdict

The retained global CGEvent probe still does not validate macOS native-control
mouse delivery. With Step 150 diagnostics present, the absence of
`window-send-event` indicates this probe did not enter `RenderWindow.sendEvent:`
for the app window.

The remaining gate is a real physical/trusted system click that logs:

```text
window-send-event -> button-mouse-down -> target-action -> button-action -> native-control-probe=makepad-click
```

