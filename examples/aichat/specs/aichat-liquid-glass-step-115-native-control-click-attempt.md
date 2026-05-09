# AI Chat Liquid Glass Step 115 Native Control Click Attempt

Date: 2026-05-09

## Run

Studio remote RunItem:

```text
makepad-example-aichat-macos-native-clear-control-probe
```

Build id:

```text
[114]
```

This build includes:

- Step 112 `event=target-action` / `event=button-action` diagnostics
- Step 113 topmost native control insertion
- Step 114 aichat `native-control-probe=makepad-click` diagnostics

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

Studio widget query from the same runnable family placed the buttons at:

```text
clear_button Button 828 564 78 36
send_button Button 914 560 44 44
```

The mapped clear-button click target was approximately:

```text
2129,786
```

## Click Attempt

System-level click command:

```text
/opt/homebrew/bin/cliclick c:2129,786
```

Additional small-area scans around that point and the vertically mirrored point
were also attempted.

## Result

No new logs appeared after the system click attempts:

- no `event=target-action`
- no `event=button-action`
- no `native-control-probe=makepad-click`

This does not validate native interactive controls. It also does not yet prove
that the AppKit button installer is wrong, because the local automation path may
not be delivering events to the Studio-launched app window. The next useful
debug step is either a human/manual click verdict on the visible app window or a
lower-level AppKit hit-test/view-hierarchy diagnostic from inside the process.
