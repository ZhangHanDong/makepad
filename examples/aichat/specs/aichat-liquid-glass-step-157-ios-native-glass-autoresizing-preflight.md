# Step 157 - iOS Native Glass Autoresizing Preflight

Date: 2026-05-10

## Change

The iOS native glass container installer applies an autoresizing mask to the
native container view:

```text
UIVisualEffectView.setAutoresizingMask:
```

Step 157 adds this selector to the iOS runtime preflight and to the selector
coverage test.

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

This closes another preflight gap in the compiled UIKit native glass skeleton.
It does not prove rotation, resize, safe-area, split-view, or Stage Manager
behavior on an iOS 26 runtime.

