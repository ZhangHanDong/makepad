# Step 126: Native Control CGEvent PID Probe

Date: 2026-05-09

## Result

macOS native controls now have a separate CGEvent process-targeted probe
runnable:

```text
makepad-example-aichat-macos-native-clear-control-cgevent-pid-probe
```

The runnable enables the existing native-control probe and sets:

```text
MAKEPAD_NATIVE_GLASS_CONTROL_CGEVENT_PROBE=pid:Clear
```

The macOS backend parses the `pid:` prefix and uses `CGEventPostToPid` with the
current process id instead of posting to the global event tap. The existing
`Clear` and `global:Clear` forms still use `CGEventPost`.

## Runtime Evidence

Studio release build `[138]` installed the native clear substrate and AppKit
button mirrors:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
```

The AppKit-side center-point hit test still resolved the native Clear button:

```text
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000043 label="Clear" point=(734.5,492.0) result_class=NativeGlassButton matches_control=true
```

The PID-targeted CGEvent probe fired:

```text
[liquid-glass] backend=apple-native-controls event=cg-event-probe mode=Process control=0000000000000043 label="Clear" local=(734.5,492.0) screen=(2004.5,1024.0)
```

No subsequent `button-mouse-down`, `target-action`, `button-action`, or
`native-control-probe=makepad-click` logs appeared.

## Meaning

`CGEventPostToPid` does not validate real native-control click delivery in the
current Studio-launched environment. Together with Step 125, this narrows the
remaining failure to system event injection, permissions, coordinate semantics,
or window-server routing rather than the AppKit native button geometry or
target/action bridge.

Step 124 remains the strongest passing event evidence: in-process AppKit
`NSEvent` mouse events posted through the application queue reach the native
button and re-enter Makepad. The physical/system click gate remains open.
