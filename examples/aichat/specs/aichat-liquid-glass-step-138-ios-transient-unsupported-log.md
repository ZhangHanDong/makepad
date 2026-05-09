# Step 138 - iOS Transient Popup Unsupported Log

Date: 2026-05-10

## Change

iOS now has a stable Phase I transient-window response when
`MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE` is enabled and a popup window is created.

The iOS backend does not claim transient native glass support yet. iOS popup
windows are currently rendered as overlays on the main `MTKView`; they do not
own a separate UIKit platform window or native glass host. Instead,
`CxOsOp::CreatePopupWindow` logs a deterministic unsupported result:

```text
[liquid-glass] transient-window=popup state=Unsupported substrate=ios-native style=clear reason=transient-platform-window-missing
```

If the current backend request is not an Apple native glass backend, it logs a
stable rejection instead:

```text
[liquid-glass] transient-window=popup state=Rejected substrate=ios-native reason=backend-not-native
```

The backend style aliases intentionally match the existing aichat startup
contract:

- `macos-native` / `apple-native-underlay` / `auto` -> `regular`
- `macos-native-clear` / `apple-native-underlay-clear` -> `clear`

## Verification

Passed:

```text
cargo check -p makepad-platform --target aarch64-apple-ios --release
git diff --check
```

Host `cargo test -p makepad-platform ios_native_glass_transient --release`
compiled the host platform test binary but ran `0` tests because `ios.rs` is
not active for the host target.

iOS test compilation with:

```text
cargo test -p makepad-platform ios_native_glass_transient --target aarch64-apple-ios --release --no-run
```

was initially blocked by an unrelated existing test-target compile error in
`platform/src/live_reload.rs`. Step 142 removes that validation blocker; the
iOS test target now compiles with `--no-run`.

## Verdict

This closes the Phase I iOS transient "stable unsupported logging" gate. It
does not implement UIKit transient native glass or prove iOS 26 runtime visual
behavior.

Still missing for full Phase I:

- physical macOS outside-click or Escape dismissal evidence,
- UIKit transient-window implementation and runtime validation,
- modal-window native glass support.
