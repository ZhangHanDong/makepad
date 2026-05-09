# Step 141 - In-Window Modal Native Glass Unsupported Gate

Date: 2026-05-10

## Change

`widgets::Modal` now exposes a default-off `native_glass` opt-in. Because the
current Modal widget is rendered inside the existing Makepad window and does
not own a separate platform window, the opt-in does not install native glass.

When a modal with `native_glass: true` is opened, it logs a stable unsupported
reason:

```text
[liquid-glass] transient-window=modal state=Unsupported reason=in-window-modal-has-no-platform-window
```

Default behavior remains unchanged:

```text
native_glass: false
```

## Verification

Passed:

```text
cargo test -p makepad-widgets modal_native_glass_unsupported_log_line_is_stable --release
cargo check -p makepad-widgets --release
cargo check -p makepad-example-aichat --release
```

## Verdict

This closes the in-window Modal observability gate: Makepad-rendered modals do
not silently claim Apple native transient glass. Native modal glass remains a
future separate-platform-window phase.
