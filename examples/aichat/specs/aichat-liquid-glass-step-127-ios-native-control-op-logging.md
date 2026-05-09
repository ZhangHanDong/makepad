# Step 127: iOS Native Control Op Logging

Date: 2026-05-09

## Result

iOS no longer silently ignores `CxOsOp::SetNativeGlassControlBatch`.

The iOS backend now validates incoming `NativeGlassControlBatch` values with
`validate_v4_10` and logs one of two stable outcomes:

```text
[liquid-glass] backend=apple-native-ios-controls state=Unsupported reason=installer-not-implemented controls_total=N controls_visible=M
[liquid-glass] backend=apple-native-ios-controls state=Rejected reason=<validation-reason> controls_total=N controls_visible=M
```

The supported validation reasons are:

```text
too-many-visible-controls
empty-visible-control-rect
```

## Meaning

This does not implement UIKit native controls. It makes the iOS control path
observable and aligned with the macOS policy: descriptor delivery and validation
can be distinguished from missing UIKit installer work.

The next UIKit control phase still needs an iOS 26 SDK/runtime validation pass
for button appearance, hit testing, accessibility ownership, and action bridging.

## Verification

```bash
cargo check -p makepad-example-aichat
cargo test -p makepad-platform native_glass -- --nocapture
```

The `native_glass` test filter passed 33 tests, including the new
`ios_native_glass_control_validation_reason_maps_errors` coverage.
