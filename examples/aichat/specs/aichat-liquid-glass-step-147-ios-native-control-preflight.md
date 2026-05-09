# Step 147 - iOS Native Control Preflight

Date: 2026-05-10

## Change

iOS native-control batch handling now performs a UIKit runtime preflight before
falling back to the installer gate.

The preflight checks:

- `UIButton`
- `UIButtonConfiguration`
- `buttonWithType:`
- `setFrame:`
- `setUserInteractionEnabled:`
- `setAccessibilityLabel:`
- `setConfiguration:`
- `addTarget:action:forControlEvents:`
- `glassButtonConfiguration`
- `clearGlassButtonConfiguration`
- `insertSubview:aboveSubview:`

Valid native-control batches now log the missing UIKit gate explicitly:

```text
[liquid-glass] backend=apple-native-ios-controls state=Unsupported reason=uikit-control-class-missing missing_class=UIButtonConfiguration missing_selector=none controls_total=N controls_visible=M
[liquid-glass] backend=apple-native-ios-controls state=Unsupported reason=uikit-control-selector-missing missing_class=none missing_selector=glassButtonConfiguration controls_total=N controls_visible=M
[liquid-glass] backend=apple-native-ios-controls state=Unsupported reason=installer-not-implemented missing_class=none missing_selector=none controls_total=N controls_visible=M
```

Invalid batches keep the existing rejection reasons:

```text
[liquid-glass] backend=apple-native-ios-controls state=Rejected reason=too-many-visible-controls controls_total=N controls_visible=M
[liquid-glass] backend=apple-native-ios-controls state=Rejected reason=empty-visible-control-rect controls_total=N controls_visible=M
```

## Verification

Passed:

```text
cargo test -p makepad-platform ios_native_glass_control --target aarch64-apple-ios --release --no-run
cargo check -p makepad-platform --target aarch64-apple-ios --release
git diff --check
```

The local iPhoneOS SDK still cannot runtime-validate iOS 26 Liquid Glass button
configuration. This step only makes that gate precise and observable.

## Verdict

This advances iOS native controls from a generic unsupported log to a real
UIKit button/configuration preflight.

It does not install `UIButton` mirrors, configure iOS 26 glass button
appearance, route `UIControlEventTouchUpInside` into
`NativeGlassControlActivatedEvent`, or validate behavior on an iOS 26 runtime.
