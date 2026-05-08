spec: task
name: "AI Chat Liquid Glass Step 33 Apple Native Underlay Naming"
tags: [makepad, aichat, liquid-glass, apple-native-underlay, diagnostics]
---

## Intent

Align the current macOS native glass route with the v4.2 route decision. The
existing native path is an Apple native underlay/substrate proof, not full
native interior Liquid Glass. Add explicit `apple-native-underlay` naming while
preserving the existing `macos-native` compatibility names.

## Decisions

- Add `AICHAT_GLASS_BACKEND=apple-native-underlay` and
  `apple-native-underlay-clear` as aliases for the current macOS native
  underlay route.
- Keep `macos-native` and `macos-native-clear` working for compatibility and
  historical validation docs.
- Add Studio run items with `apple-native-underlay` names.
- Update non-compatibility logs to say `apple-native-underlay`.
- Keep the v1 State 4 compatibility log as `substrate=macos-native`.
- Do not change native AppKit installation behavior.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-33-apple-native-underlay-naming.spec`
- `examples/aichat/src/main.rs`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_window.rs`
- `makepad.splash`

### Forbidden

- Do not remove `macos-native` compatibility backend values.
- Do not change `NSGlassEffectView` or `NSGlassEffectContainerView` setup.
- Do not add ShaderBackdrop behavior.
- Do not claim full native Liquid Glass.

## Acceptance Criteria

Scenario: aichat accepts explicit AppleNativeUnderlay backend aliases
Test: `cargo test -p makepad-example-aichat aichat_glass_backend_env_contract`
Given the v4.2 route decision reserves full native naming for future interleave
When backend env parsing is tested
Then both `apple-native-underlay` values resolve to the current macOS native styles
And the old `macos-native` compatibility values still resolve

Scenario: startup waits for platform native result without false unavailable warning
Test: `cargo test -p makepad-example-aichat aichat_startup_glass_resolution_waits_for_platform_result`
Given native backend activation is confirmed asynchronously by the platform
When startup resolves an explicit native or underlay request
Then it stays on shader temporarily without logging an unavailable warning

Scenario: platform env parsing accepts underlay aliases
Test: `rg "apple-native-underlay|apple-native-underlay-clear" platform/src/os/apple/macos/macos.rs`
Given Studio launches the underlay run items
When the macOS platform reads `AICHAT_GLASS_BACKEND`
Then the platform native substrate request recognizes the new aliases

Scenario: non-compatibility logs use underlay terminology
Test: `rg "backend=apple-native-underlay|app-substrate=apple-native-underlay" examples/aichat/src/main.rs platform/src/os/apple/macos/macos_window.rs`
Given State 4 compatibility logs remain historical
When newer app and batch diagnostics are inspected
Then they distinguish the current route as AppleNativeUnderlay

Scenario: Studio exposes explicit underlay run items
Test: `rg "makepad-example-aichat-apple-native-underlay|apple-native-underlay-clear" makepad.splash`
Given the user launches via Studio runnable items
When run items are listed
Then explicit underlay names are available without removing the old macOS names

Scenario: affected crates compile
Test: `cargo check -p makepad-example-aichat`
Given only naming aliases and logs changed
When aichat is checked
Then the existing glass backend wiring still compiles
