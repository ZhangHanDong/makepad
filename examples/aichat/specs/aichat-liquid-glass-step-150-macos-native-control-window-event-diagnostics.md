# Step 150 - macOS Native Control Window Event Diagnostics

Date: 2026-05-10

## Change

The macOS `RenderWindow` subclass now logs mouse events in `sendEvent:` when
the native-control probe is enabled.

The log shape is:

```text
[liquid-glass] backend=apple-native-controls event=window-send-event type=NSLeftMouseDown point=(x,y) hit=NativeGlassButton
```

This diagnostic is gated by `AICHAT_NATIVE_CONTROL_PROBE` through the existing
native-control probe guard, so normal windows do not log every mouse event.

## Purpose

The physical-click gate needs to distinguish three cases:

1. The mouse event never enters the app/window.
2. The event enters `NSWindow.sendEvent:` but hit-testing resolves to the Metal
   view or another view.
3. The event enters `NSWindow.sendEvent:`, hit-testing resolves to
   `NativeGlassButton`, and the button still does not receive `mouseDown:`.

Step 150 gives the next manual click run enough evidence to identify which
case is happening.

## Verification

Passed:

```text
cargo check -p makepad-platform --release
git diff --check
```

## Verdict

This is diagnostic coverage only. It does not close the physical mouse-click
gate until a real user click is captured with `window-send-event`,
`button-mouse-down`, `target-action`, `button-action`, and
`native-control-probe=makepad-click`.
