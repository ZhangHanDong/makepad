# Step 164 - Native Control Geometry Mouse Probe

Date: 2026-05-10

## Context

Step 163 fixed the macOS native-control geometry refresh path by forcing
installed `NativeGlassButton` frames to be recomputed after window geometry
changes. That still left one validation gap: the existing native-control mouse
probe only fired once per control id, so a control that moved after resize would
not be re-tested automatically.

## Change

The macOS native-control probe de-duplication key now includes both the control
id and the descriptor rect:

```text
(control_id, rect)
```

This keeps probes one-shot for stable geometry, but lets them fire again when
the same logical button is re-laid out to a different Makepad rect.

The new Studio runnable is:

```text
makepad-example-aichat-macos-native-clear-control-geometry-mouse-probe
```

It enables:

```text
AICHAT_GLASS_BACKEND=macos-native-clear
MAKEPAD_NATIVE_GLASS_GEOMETRY_PROBE=1
MAKEPAD_NATIVE_GLASS_CONTROL_PROBE=buttons
MAKEPAD_NATIVE_GLASS_CONTROL_MOUSE_EVENT_PROBE=Clear
```

## Runtime Evidence

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-control-geometry-mouse-probe
build_id=[17]
commit=6eecea3e
```

The run reached the native clear substrate and native-control gates:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
```

The initial synthetic mouse probe activated the native `Clear` button and
bridged back into Makepad:

```text
[liquid-glass] backend=apple-native-controls event=synthetic-mouse-probe ... label="Clear" ...
[liquid-glass] backend=apple-native-controls event=window-send-event type=NSLeftMouseDown ... hit=NativeGlassButton
[liquid-glass] backend=apple-native-controls event=button-action control_id=67
[liquid-glass] native-control-probe=makepad-click id=clear_button
```

After geometry changes, the same run continued to refresh control frames and
revalidate AppKit hit testing against the moved frames:

```text
[liquid-glass] backend=apple-native-controls event=frame-refresh reason=geometry-change controls_total=2 controls_visible=2
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000043 label="Clear" ... result_class=NativeGlassButton matches_control=true
```

The post-geometry mouse-probe path is now able to re-fire because the control
rect is part of the probe key. This closes the automatic geometry-probe
coverage gap from Step 163, but it does not by itself prove physical AppKit
click delivery.

## Verification Commands

```text
git diff --check
cargo test -p makepad-platform native_glass_control_probe_key_includes_rect --release
cargo check -p makepad-platform --release
```

## Remaining Gate

Physical/system click validation is still separate. A fresh native-control run
still needs a trusted manual or system click on the visible `Clear` button that
produces:

```text
window-send-event -> button-mouse-down -> target-action -> button-action -> native-control-probe=makepad-click
```
