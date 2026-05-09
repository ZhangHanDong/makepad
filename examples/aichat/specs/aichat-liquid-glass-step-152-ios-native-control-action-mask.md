# Step 152 - iOS Native Control Action Mask

Date: 2026-05-10

## Change

The UIKit native-control installer now names the UIKit raw constants it uses:

```rust
pub const UI_BUTTON_TYPE_SYSTEM: i64 = 1;
pub const UI_CONTROL_EVENT_TOUCH_UP_INSIDE: u64 = 1 << 6;
```

`install_native_glass_control_batch` now passes those constants to:

```text
UIButton.buttonWithType:
UIButton.addTarget:action:forControlEvents:
```

This makes the iOS native-control event wiring explicit: mirrored glass buttons
are created as system buttons and dispatch `nativeGlassControlAction:` on
`touchUpInside`.

## Tests

Added focused helper tests in `platform/src/os/apple/ios/ios_app.rs`:

```text
ios_native_glass_control_style_line_records_configuration
ios_native_glass_control_installer_uses_touch_up_inside_action_mask
```

Host `cargo test -p makepad-platform ... --release` filtered these tests out
because the iOS backend is not compiled into the host test target. The valid
checks for this step are the iOS target build gates below.

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

This improves the compiled UIKit native-control skeleton and documents the
intended touch-up-inside action path. It does not replace iOS 26 runtime
validation: real device/simulator evidence is still required for class
availability, glass button configuration selectors, visual output, hit testing,
and action delivery.

