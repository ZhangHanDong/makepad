# AI Chat Liquid Glass Step 118 Native Control Hit-Test Runtime

Date: 2026-05-09

## Run

Studio remote RunItem:

```text
makepad-example-aichat-macos-native-clear-control-probe
```

Build id:

```text
[116]
```

This build includes:

- Step 116 `NativeGlassButton` subclass diagnostics
- Step 117 probe-gated Metal view diagnostics

## Startup Evidence

Relevant Studio/child-app log lines:

```text
[liquid-glass] native-control-probe=buttons-enabled ids=clear_button,send_button
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

Quartz window bounds:

```text
makepad-example-aichat { X = 1279; Y = 215; Width = 882; Height = 686; }
```

## Click Attempts

System-level click:

```text
/opt/homebrew/bin/cliclick c:2129,786
```

Window-center system click:

```text
/opt/homebrew/bin/cliclick c:1720,558
```

Studio bridge click:

```text
{"Click":{"build_id":[116],"x":867,"y":582}}
```

The bridge also confirmed the expected Makepad button rects:

```text
clear_button Button 828 564 78 36
send_button Button 914 560 44 44
```

## Log Query

Post-click log query:

```text
{"QueryLogs":{"build_id":[116],"pattern":"liquid-glass","since_index":8154,"live":false}}
```

Result:

```text
only index 8154, the existing startup app-substrate log
```

No post-click diagnostics appeared:

- no `event=accepts-first-mouse`
- no `event=button-hit-test`
- no `event=button-mouse-down`
- no `event=metal-view-hit-test`
- no `event=metal-view-mouse-down`
- no `event=target-action`
- no `event=button-action`
- no `native-control-probe=makepad-click`

## Result

This is not a passing native-control validation. The startup installer still
proves the AppKit controls are created, but the local automated click path did
not produce AppKit, Metal-view, or Makepad button events.

The next useful verification must be one of:

- a human/manual click verdict on the visible app window while watching the
  Step 116/117 diagnostics
- an in-process AppKit view-hierarchy dump that proves where the native button
  sits relative to the Metal view and window content view
