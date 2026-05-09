# Step 153 - iOS Native Control Enabled State

Date: 2026-05-10

## Change

The UIKit native-control installer now mirrors `NativeGlassControlDescriptor`
`enabled` state into the native `UIButton` itself:

```text
UIButton.setUserInteractionEnabled:
UIButton.setEnabled:
```

The iOS native-control selector preflight now also requires:

```text
UIButton.setEnabled:
```

This matters because `setUserInteractionEnabled:` only controls event delivery.
`setEnabled:` controls the button's native enabled/disabled state, including
visual state and accessibility semantics.

## Verification

Passed:

```text
rustfmt --edition 2021 platform/src/os/apple/ios/ios.rs platform/src/os/apple/ios/ios_app.rs
cargo check -p makepad-platform --target aarch64-apple-ios --release
cargo test -p makepad-platform --target aarch64-apple-ios --release --no-run
git diff --check
```

The iOS target commands still report existing unrelated warnings in
`apple_game_input.rs`, `metal.rs`, and `web_socket.rs`.

## Verdict

The compiled UIKit native-control skeleton now maps both interaction delivery
and native enabled state. Real iOS 26 runtime validation is still required for
visual disabled styling, hit testing, VoiceOver behavior, and action delivery.

