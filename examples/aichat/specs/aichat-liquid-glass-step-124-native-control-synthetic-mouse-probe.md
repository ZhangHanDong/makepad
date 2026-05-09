# Step 124: Native Control Synthetic Mouse Probe

Date: 2026-05-09

## Result

macOS native controls now have an explicit synthetic mouse-event probe runnable:

```text
makepad-example-aichat-macos-native-clear-control-mouse-probe
```

The runnable enables the existing native-control probe and sets:

```text
MAKEPAD_NATIVE_GLASS_CONTROL_MOUSE_EVENT_PROBE=Clear
```

The macOS backend posts an AppKit `NSLeftMouseDown` / `NSLeftMouseUp` pair to
the `NSApplication` event queue at the matching native control's center point.
A per-window guard prevents repeating the synthetic mouse probe for the same
control when native control batches are reinstalled.

## Runtime Evidence

Studio release build `[133]` logged the synthetic mouse path:

```text
[liquid-glass] backend=apple-native-controls event=synthetic-mouse-probe control=0000000000000043 label="Clear" point=(734.5,492.0)
[liquid-glass] backend=apple-native-controls event=button-mouse-down window_point=(740.4,494.9)
[liquid-glass] backend=apple-native-controls event=target-action window_index=0 window_generation=0 control_id=67
[liquid-glass] backend=apple-native-controls event=button-action control_id=67
[liquid-glass] native-control-probe=makepad-click id=clear_button
```

This proves an AppKit mouse event posted through the application event queue can
reach the installed `NativeGlassButton`, invoke its `mouseDown:` path, trigger
the AppKit target/action bridge, and re-enter Makepad's existing button action
handler.

## Non-Fix Attempt

An earlier attempt used synchronous `sendEvent:` for `NSLeftMouseDown` before
queuing mouse up. That reached `button-mouse-down` but did not continue to
target/action because AppKit's button tracking path waited for the corresponding
mouse-up event. The retained implementation posts both events to the application
queue instead.

## Remaining Limit

This is still not a physical user/system click. The remaining validation gate is
a real click from the OS/hardware or a reliable system automation path producing
the same sequence without synthetic in-process event construction.

