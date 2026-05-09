# Step 159 - iOS Native Panel Frame Log

Date: 2026-05-10

## Change

The UIKit native glass panel installer now logs each installed panel's Makepad
descriptor frame, UIKit frame relative to the native container, style, tint,
corner radius, z-order, and visibility.

The log shape is:

```text
[liquid-glass] backend=apple-native-ios native-panel-frame container_rect=(...) panel=... makepad=(...) ui=(...) style=... tint=(...) corner_radius=... z_order=... visible=...
```

This mirrors the intent of macOS native panel frame logs and gives future iOS
26 runtime validation concrete geometry/tint evidence to compare against visual
screenshots.

## Tests

Added helper coverage:

```text
ios_native_glass_panel_frame_snapshot_line_records_geometry
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

This is runtime observability for the compiled UIKit native panel skeleton. It
does not prove real iOS 26 visual output, frame alignment, tint fidelity,
corner-radius behavior, safe-area behavior, rotation, split-view behavior, or
Stage Manager behavior.

