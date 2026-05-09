# Step 149 - macOS Native Control Accessibility Press Probe

Date: 2026-05-10

## Change

macOS native controls now have an explicit accessibility activation probe:

```text
MAKEPAD_NATIVE_GLASS_CONTROL_ACCESSIBILITY_PRESS_PROBE=Clear
```

When the probe matches an installed native control, the platform calls
`accessibilityPerformPress` on the `NativeGlassButton` and logs both the probe
attempt and the returned boolean.

Studio exposes this as:

```text
makepad-example-aichat-macos-native-clear-control-accessibility-press-probe
```

## Runtime Evidence

Studio release build `[2]` logged:

```text
[liquid-glass] backend=apple-native-controls event=accessibility-press-probe control=0000000000000043 label="Clear"
[liquid-glass] backend=apple-native-controls event=target-action window_index=0 window_generation=0 control_id=67
[liquid-glass] backend=apple-native-controls event=accessibility-press-result control=0000000000000043 label="Clear" result=false
[liquid-glass] backend=apple-native-controls event=button-action control_id=67
[liquid-glass] native-control-probe=makepad-click id=clear_button
```

The native control batch was installed before the probe:

```text
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
```

## Verification

Passed:

```text
cargo test -p makepad-platform native_glass_control --release
cargo check -p makepad-platform --release
git diff --check
```

## Verdict

This proves the macOS native accessibility activation path can trigger the same
target/action bridge used by AppKit native controls, even though
`accessibilityPerformPress` returned `false` on the probed button.

It does not close the physical/system mouse-click gate. A real user click still
must produce `button-mouse-down`, `target-action`, `button-action`, and
`native-control-probe=makepad-click` before native interactive controls can be
called complete.
