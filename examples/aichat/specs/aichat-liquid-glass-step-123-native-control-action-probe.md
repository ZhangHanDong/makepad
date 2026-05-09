# Step 123: Native Control Action Probe

Date: 2026-05-09

## Result

macOS native controls now have an explicit action-bridge probe runnable:

```text
makepad-example-aichat-macos-native-clear-control-action-probe
```

The runnable enables the existing native-control probe and sets:

```text
MAKEPAD_NATIVE_GLASS_CONTROL_PERFORM_CLICK_PROBE=Clear
```

The macOS backend only runs this probe when the environment variable matches a
native control label, or when it is set to `1`, `true`, `on`, or `all`. A
per-window guard prevents repeating the programmatic click for the same control
when native control batches are reinstalled.

## Runtime Evidence

Studio release build `[130]` logged the full programmatic action chain:

```text
[liquid-glass] backend=apple-native-controls event=perform-click-probe control=0000000000000043 label="Clear"
[liquid-glass] backend=apple-native-controls event=target-action window_index=0 window_generation=0 control_id=67
[liquid-glass] backend=apple-native-controls event=button-action control_id=67
[liquid-glass] native-control-probe=makepad-click id=clear_button
```

The same build reinstalled the native control batch later, but did not repeat
`perform-click-probe` for `Clear`, confirming the once-per-control guard.

## Meaning

This proves the native AppKit target/action bridge can reach Makepad's
`NativeGlassControlActivatedEvent`, convert into `ButtonAction::Clicked`, and
trigger the existing aichat `clear_button` handler path.

## Remaining Limit

This is still not a full user/system click validation. `performClick:` proves
the action bridge, but not physical mouse/touch event delivery. The remaining
native interactive-control gate is a real click producing:

- `event=button-mouse-down`
- `event=target-action`
- `event=button-action`

