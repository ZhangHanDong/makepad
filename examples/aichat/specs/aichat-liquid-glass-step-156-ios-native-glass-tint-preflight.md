# Step 156 - iOS Native Glass Tint Preflight

Date: 2026-05-10

## Change

The iOS native glass panel installer converts `NativeGlassPanelDescriptor.tint`
into a native `UIColor` through:

```text
UIColor.colorWithRed:green:blue:alpha:
```

Step 156 adds this dependency to the iOS runtime preflight:

```text
required class: UIColor
required selector: colorWithRed:green:blue:alpha:
```

This keeps the dynamic UIKit backend honest: if tint color creation is not
available, the backend fails during preflight with the same class/selector
diagnostic path used by the rest of the native glass installer.

## Verification

Passed:

```text
rustfmt --edition 2021 platform/src/os/apple/ios/ios.rs
cargo check -p makepad-platform --target aarch64-apple-ios --release
cargo test -p makepad-platform --target aarch64-apple-ios --release --no-run
git diff --check
```

The iOS target commands still report existing unrelated warnings in
`apple_game_input.rs`, `metal.rs`, and `web_socket.rs`.

## Verdict

This closes a preflight gap for the compiled UIKit panel skeleton. It does not
prove iOS 26 runtime visual output, tint fidelity, or color-space correctness
on device.

