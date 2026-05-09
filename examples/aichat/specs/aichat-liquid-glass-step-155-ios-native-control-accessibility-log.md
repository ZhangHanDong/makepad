# Step 155 - iOS Native Control Accessibility Log

Date: 2026-05-10

## Change

The UIKit native-control installer already mirrors
`NativeGlassControlDescriptor.label` into each native `UIButton` with:

```text
setTitle:forState:
setAccessibilityLabel:
```

Step 155 adds a stable runtime log line for the accessibility mirror:

```text
[liquid-glass] backend=apple-native-ios-controls accessibility-label control=... label=...
```

This brings iOS native-control observability closer to the macOS native button
path, where accessibility label mirroring is also logged.

## Tests

Added helper coverage:

```text
ios_native_glass_control_accessibility_line_records_label
```

## Verification

Passed:

```text
rustfmt --edition 2021 platform/src/os/apple/ios/ios_app.rs
cargo check -p makepad-platform --target aarch64-apple-ios --release
cargo test -p makepad-platform --target aarch64-apple-ios --release --no-run
git diff --check
```

The iOS target commands still report existing unrelated warnings in
`apple_game_input.rs`, `metal.rs`, and `web_socket.rs`.

## Verdict

This is observability for the compiled UIKit native-control skeleton. It does
not prove real iOS 26 VoiceOver behavior, focus order, duplicate-label policy,
or runtime action delivery.

