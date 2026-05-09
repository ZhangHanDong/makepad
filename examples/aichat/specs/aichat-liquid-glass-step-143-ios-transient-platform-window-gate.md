# Step 143 - iOS Transient Platform Window Gate

Date: 2026-05-10

## Change

The iOS transient popup native-glass probe now reports the precise missing
primitive:

```text
[liquid-glass] transient-window=popup state=Unsupported substrate=ios-native style=clear reason=transient-platform-window-missing
```

Current iOS popup windows are not separate UIKit platform windows. They are
Makepad popup metadata plus an overlay draw pass on the main `MTKView`. That
means there is no transient `UIWindow` or popup-local `UIVisualEffectView` host
where native Liquid Glass can be installed.

This replaces the broader `transient-installer-not-implemented` reason so Phase
I can distinguish two separate tasks:

- add a real transient UIKit window/host primitive,
- then install popup/modal native glass into that primitive.

## Verification

Passed:

```text
cargo test -p makepad-platform ios_native_glass_transient --target aarch64-apple-ios --release --no-run
cargo check -p makepad-platform --target aarch64-apple-ios --release
git diff --check
```

## Verdict

This does not implement UIKit transient native glass. It makes the remaining
blocker explicit and test-covered: iOS needs a separate transient platform
window/host before popup/modal native glass can be installed.
