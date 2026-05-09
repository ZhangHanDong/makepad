# Step 125: Native Control CGEvent Probe

Date: 2026-05-09

## Result

macOS native controls now have an explicit CGEvent system-click probe runnable:

```text
makepad-example-aichat-macos-native-clear-control-cgevent-probe
```

The runnable enables the existing native-control probe and sets:

```text
MAKEPAD_NATIVE_GLASS_CONTROL_CGEVENT_PROBE=Clear
```

The macOS backend converts the matching native control's AppKit center point to
screen coordinates and posts a `kCGEventLeftMouseDown` / `kCGEventLeftMouseUp`
pair with `CGEventPost`.

## Runtime Evidence

Studio release build `[135]` logged the flipped-coordinate CGEvent attempt:

```text
[liquid-glass] backend=apple-native-controls event=cg-event-probe control=0000000000000043 label="Clear" local=(734.5,492.0) screen=(2004.5,1024.0) quartz=(2004.5,416.0)
```

No subsequent native-control click logs appeared.

Studio release build `[136]` logged the retained screen-coordinate CGEvent
attempt:

```text
[liquid-glass] backend=apple-native-controls event=cg-event-probe control=0000000000000043 label="Clear" local=(734.5,492.0) screen=(2004.5,1024.0)
```

Again, no subsequent `button-mouse-down`, `target-action`, `button-action`, or
`native-control-probe=makepad-click` logs appeared.

## Meaning

The CGEvent probe is a useful retained diagnostic entry point, but it does not
currently validate physical/system click delivery in this environment. The
failure could be caused by Accessibility/Input Monitoring permissions, global
coordinate conversion, or Studio-launched window routing.

Step 124 remains the strongest passing event evidence: in-process AppKit
`NSEvent` mouse events posted through the application queue reach the native
button and re-enter Makepad. Step 125 keeps the real system-click gate open.

