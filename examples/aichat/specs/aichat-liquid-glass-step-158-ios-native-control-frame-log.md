# Step 158 - iOS Native Control Frame Log

Date: 2026-05-10

## Change

The UIKit native-control installer now logs each native control's descriptor
frame and resulting UIKit frame before calling `UIButton.setFrame:`.

The log shape is:

```text
[liquid-glass] backend=apple-native-ios-controls native-control-frame control=... kind=... label=... makepad=(x,y,w,h) ui=(x,y,w,h) z_order=... enabled=... visible=...
```

This gives future iOS 26 runtime validation the same kind of geometry evidence
that macOS native controls already expose.

## Tests

Added helper coverage:

```text
ios_native_glass_control_frame_snapshot_line_records_geometry
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

This is runtime observability for the compiled UIKit native-control skeleton.
It does not prove real iOS 26 frame alignment, safe-area behavior, rotation,
split-view behavior, hit testing, or touch delivery.

