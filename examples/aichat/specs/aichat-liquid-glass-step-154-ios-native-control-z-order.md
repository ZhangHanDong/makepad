# Step 154 - iOS Native Control Z Order

Date: 2026-05-10

## Change

The UIKit native-control installer now uses a shared helper to filter visible
controls and sort them by `NativeGlassControlDescriptor.z_order`.

During installation, each newly created native `UIButton` is inserted above the
previous native insertion anchor:

```text
MTKView -> z0 UIButton -> z1 UIButton -> ...
```

This avoids repeatedly inserting every native control directly above `MTKView`,
which can make later controls sit below earlier native siblings depending on
UIKit insertion semantics.

## Tests

Added helper coverage:

```text
ios_native_glass_visible_controls_are_sorted_by_z_order
```

The test verifies that hidden controls are excluded and visible controls are
ordered low-to-high by `z_order` before the installer applies UIKit subview
insertion.

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

The compiled UIKit native-control skeleton now has deterministic native sibling
ordering for overlapping controls. Real iOS 26 runtime validation is still
required to confirm UIKit visual stacking, hit testing, and touch delivery.

