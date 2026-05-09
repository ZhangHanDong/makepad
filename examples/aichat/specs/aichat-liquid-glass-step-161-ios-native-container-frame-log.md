# Step 161 - iOS Native Container Frame Log

Date: 2026-05-10

## Change

The UIKit native glass installer now logs the native container frame before
creating the `UIVisualEffectView` container:

```text
[liquid-glass] backend=apple-native-ios native-container-frame container=... makepad=(...) ui=(...) spacing=... panels=...
```

This complements Step 159 panel frame logs and Step 160 container spacing logs
so future iOS 26 validation can compare the full container/panel geometry chain.

## Tests

Added helper coverage:

```text
ios_native_glass_container_frame_snapshot_line_records_geometry
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

This is runtime observability for iOS native container geometry. It does not
prove real iOS 26 safe-area behavior, rotation, split-view behavior, Stage
Manager behavior, or UIKit visual alignment.

