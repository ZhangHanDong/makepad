# Step 148 - iOS Native Control Installer Skeleton

Date: 2026-05-10

## Change

iOS native-control batch handling now proceeds past preflight into a UIKit
installer skeleton when the required classes and selectors are present.

The installer:

- creates `UIButton` mirrors for visible `NativeGlassControlDescriptor` buttons,
- selects `UIButtonConfiguration.glass()` for `NativeGlassStyle::Regular`,
- selects `UIButtonConfiguration.clearGlass()` for `NativeGlassStyle::Clear`,
- sets the button title and accessibility label from the descriptor label,
- places the native button above the MTK view,
- wires `UIControl.Event.touchUpInside` to
  `NativeGlassControlActivatedEvent` through `IosNativeGlassControlTarget`,
- preserves the existing rejection path for invalid batches.

Expected runtime logs on an iOS 26 runtime:

```text
[liquid-glass] backend=apple-native-ios-controls button-style control=... label="..." configuration=glassButtonConfiguration style=Regular
[liquid-glass] backend=apple-native-ios-controls state=Installed reason=installed-uikit-buttons controls_total=N controls_visible=M
[liquid-glass] backend=apple-native-ios-controls event=target-action window_index=... window_generation=... control_id=...
```

## Verification

Passed:

```text
cargo test -p makepad-platform ios_native_glass_control --target aarch64-apple-ios --release --no-run
cargo check -p makepad-platform --target aarch64-apple-ios --release
cargo check -p makepad-platform --release
git diff --check
```

## Verdict

This is the first compiled UIKit native-control installer skeleton.

It is not a runtime completion claim. The local environment still lacks an iOS
26 runtime validation path, so real `UIButtonConfiguration.glass()` appearance,
hit testing, accessibility ownership, and `touchUpInside` delivery remain
unproven until this runs on iOS 26.
