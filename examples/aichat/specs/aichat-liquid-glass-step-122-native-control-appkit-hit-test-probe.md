# Step 122: Native Control AppKit Hit-Test Probe

Date: 2026-05-09

## Result

macOS native controls now have an AppKit-side geometry validation path that does
not depend on Studio framebuffer screenshots or Studio remote input
coordinates.

After each visible `NativeGlassButton` is inserted into the AppKit container,
the macOS backend calls `hitTest:` on the container at the native control's
center point and logs whether the returned view is the same native control.

## Runtime Evidence

Studio release build `[127]` for
`makepad-example-aichat-macos-native-clear-control-probe` logged:

```text
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000043 label="Clear" point=(734.5,492.0) result_class=NativeGlassButton matches_control=true
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000044 label="↑" point=(803.5,488.0) result_class=NativeGlassButton matches_control=true
```

This proves AppKit hit testing reaches the installed native button views at
their center points and that sibling z-order is sufficient for AppKit to select
the native controls over the Metal view at those points.

The same build also logged:

```text
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
```

## Remaining Limits

This does not yet prove end-to-end user/system click delivery. The next passing
gate still needs a real click to produce:

- `event=button-mouse-down`
- `event=target-action`
- `event=button-action`

The probe is intentionally geometry-focused: it proves AppKit's own hit-test
resolution for the installed native controls, not full input event routing.

