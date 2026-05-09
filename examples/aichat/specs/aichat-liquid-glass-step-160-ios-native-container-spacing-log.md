# Step 160 - iOS Native Container Spacing Log

Date: 2026-05-10

## Change

The UIKit native glass container installer now logs the `spacing` value applied
to `UIGlassContainerEffect`:

```text
[liquid-glass] backend=apple-native-ios native-container-spacing container=... spacing=...
```

This mirrors the macOS `native-container-spacing` evidence and makes the
container morph parameter visible during future iOS 26 runtime validation.

## Tests

Added helper coverage:

```text
ios_native_glass_container_spacing_line_records_spacing
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

This is runtime observability for iOS native container morph spacing. It does
not prove real iOS 26 morphing behavior, animation quality, or resize
synchronization.

