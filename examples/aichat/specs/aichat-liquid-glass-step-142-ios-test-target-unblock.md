# Step 142 - iOS Native Glass Test Target Unblock

Date: 2026-05-10

## Change

The iOS native glass test target can now compile with `--no-run`.

`platform/src/live_reload.rs` had three pure hot-reload helper functions gated
to desktop targets. That was valid for the production hot-reload path, but it
blocked iOS test-target compilation because the host-side hot-reload tests are
still compiled for the iOS test binary.

The helpers are now available to test compilation on iOS targets and marked
with `#[allow(dead_code)]` so non-desktop library checks do not add warnings.
No iOS runtime behavior or UIKit native glass installer behavior changes in
this step.

## Verification

Passed:

```text
cargo test -p makepad-platform excludes_platform_script_and_draw_manifests_from_hot_reload --release
cargo test -p makepad-platform ios_native_glass_transient --target aarch64-apple-ios --release --no-run
cargo check -p makepad-platform --target aarch64-apple-ios --release
git diff --check
```

The iOS test-target command compiles the test executable but does not run it on
an iOS device or simulator.

## Verdict

This closes the validation infrastructure gap from Step 138. It proves that the
iOS native glass transient unsupported-log tests compile for the iOS target.

It does not prove iOS 26 UIKit class availability, selector availability,
native glass visual output, safe-area behavior, rotation behavior, keyboard
behavior, split-view behavior, or Stage Manager behavior.
