# Issue: WindowBackdrop != None crashes on macOS 15.5

Status: open
Severity: blocker for v3 liquid-glass (real backdrop blur)
Discovered: 2026-04-27 while wiring `WindowBackdrop.Blur` into aichat
File: `platform/src/os/apple/macos/macos_window.rs:532`

## Symptom

Setting `window.backdrop: WindowBackdrop.Blur` (or any non-None value) on a Makepad `Window` causes the app to SIGABRT during the first timer-driven `set_window_visuals` callback.

```
assertion failure: "place == NSWindowBelow || place == NSWindowAbove" -> %llu
0   libsystem_kernel.dylib  __abort_with_payload
3   libsystem_c.dylib       _os_crash_msg
4   AppKit                  -[NSView addSubview:positioned:relativeTo:] + 988
5   makepad-example-aichat-agent2app  msg_send invoke
7   makepad-platform        macos_window::set_window_visuals + 4952
```

Reproducible on macOS 15.5 (24F74) / Apple M4 Pro / Darwin 24.5.0.

## Root cause

`platform/src/os/apple/macos/macos_window.rs:532`:

```rust
let () = msg_send![
    self.view,
    addSubview: effect_view
    positioned: 0i64
    relativeTo: nil
];
```

The `0i64` argument is invalid. macOS's `NSView.addSubview(_:positioned:relativeTo:)` documents the second argument as `NSWindowOrderingMode`, an enum with exactly two valid values:

- `NSWindowAbove = 1`
- `NSWindowBelow = -1`

`0` is `NSWindowOut`, which Apple uses elsewhere (NSWindow z-ordering) but **not** as a valid value for this call. macOS 15.x added a hard assertion that rejects anything except `±1`. The code path likely worked silently on older macOS that didn't enforce the assertion.

This code was never exercised in tree until aichat became the first call site to set `window.backdrop != None`.

## Fix

Use `NSWindowBelow = -1` so the NSVisualEffectView is inserted as the first (z-bottom) subview, which is the right z-order for a backdrop layer.

```rust
const NS_WINDOW_BELOW: i64 = -1;
let () = msg_send![
    self.view,
    addSubview: effect_view
    positioned: NS_WINDOW_BELOW
    relativeTo: nil
];
```

## Secondary concern (post-fix verification needed)

Even after `0` → `-1`, the visual effect view is still added as a **subview of `self.view`** (the metal-backed NSView). On layer-backed NSViews, sublayer order is host-layer's own content + sublayers in array order. When `self.view` has a CAMetalLayer as its content layer, that metal content + a NSVisualEffectView sublayer's CALayer end up as siblings — but the host's own metal content sometimes draws on top regardless of subview order.

If the fix doesn't produce the expected backdrop blur after the assertion is gone, the next architectural fix is:

1. Set `window.contentView = NSVisualEffectView` (replacing the current setup)
2. Add the metal-backed NSView (`self.view`) as a subview of the visual effect view
3. Make the metal layer transparent (already does via `setOpaque: NO`)

This second fix is more invasive but is the canonical NSVisualEffectView pattern used by Finder sidebar, Spotlight, NotificationCenter, and the macOS Sequoia "liquid glass" UI.

## Workaround applied to aichat

`examples/aichat-agent2app/src/main.rs` `main_window` declaration: the two `window.backdrop` / `window.backdrop_intensity` lines are commented out with a pointer to this issue. v2 visual ships without real backdrop blur; v3 unlocks once this is fixed.

## Plan to fix

Two options:

| Option | Scope | Risk |
|---|---|---|
| A. One-line fix `0i64` → `-1i64` | `platform/src/os/apple/macos/macos_window.rs` | low — fixes the crash, may or may not produce visible blur depending on layer ordering |
| B. One-line fix + verify visual + (if needed) restructure to set NSVisualEffectView as `contentView` | platform + plumbing | medium — touches core window setup |

Recommended sequence: A first; if A unblocks the crash but blur isn't visible, then B.

## Verification

After fix, in aichat:

1. Re-enable `window.backdrop: WindowBackdrop.Blur` and `window.backdrop_intensity: 1.0` in `examples/aichat-agent2app/src/main.rs`
2. Run over a real desktop wallpaper
3. Confirm: wallpaper visible behind glass panels is **blurred**, not crisp
4. Confirm: app does not crash on launch
5. Lower per-panel `tint_alpha` (currently bumped to 0.86–0.94 to compensate for crisp transmission) back toward the v2 spec table values (0.55–0.78) — with real blur, lower alpha works again
