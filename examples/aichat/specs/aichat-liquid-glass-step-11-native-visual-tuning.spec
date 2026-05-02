spec: task
name: "AI Chat Liquid Glass Step 11 Native Visual Tuning"
tags: [makepad, aichat, liquid-glass, macos-native, studio-validation]
---

## Intent

Record the first successful Studio visual validation for aichat native Liquid
Glass, reduce Makepad-rendered overlay decoration so the Apple-native material
is easier to see, and prevent identical native glass batches from being
reinstalled and re-logged every frame.

## Decisions

- Keep the v4.1 native backend model unchanged: one container per window, four
  aichat panels, passthrough hit testing, and Makepad content above native glass.
- Treat the user-reported Studio visual pass on 2026-05-02 as the first manual
  validation record for regular and clear native styles.
- Lower `GlassPanelPreset::NativeOverlay` tint, border, highlight, noise, and
  halo contribution; native glass should come from AppKit, not from a heavy
  Makepad overlay.
- Make `macos-native-clear` lighter than regular native overlay, because clear
  style should expose the native material with less extra tint.
- Cache the last macOS native batch result at the platform layer so repeated
  identical descriptors return the previous result without rebuilding AppKit
  views or emitting duplicate install logs.

## Constraints

- Must not change native descriptor schema.
- Must not change NSGlassEffectView raw style defaults.
- Must not introduce native + shader backdrop mixing.
- Must keep non-native shader fallback behavior unchanged.
- Must keep Studio validation runnable through RunItem, not raw `cargo run`.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-11-native-visual-tuning.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
- `examples/aichat/src/main.rs`
- `platform/src/os/apple/macos/macos_window.rs`

### Forbidden

- `widgets/**`
- `platform/src/os/apple/ios/**`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`

### Out of Scope

- Fullscreen, Stage Manager, multiple-display, popup/modal, and runtime
  substrate switching behavior.
- ShaderBackdrop blur/refraction work.
- Native interactive glass buttons.

## Acceptance Criteria

Scenario: manual Studio validation is recorded
Test: `rg "2026-05-02|manual visual pass|panels_installed=4 panels_failed=0" examples/aichat/specs/aichat-liquid-glass-v4.1-*.md`
Given regular and clear native targets have been run through Studio
When the validation documents are read
Then they record State 4, four installed panels, and the manual visual pass

Scenario: native overlay is lighter than shader overlay
Test: `cargo test -p makepad-example-aichat aichat_native -- --nocapture`
Given `GlassPanelPreset::NativeOverlay` is used
When opacity values are calculated
Then tint, border, highlight, noise, and halo remain lower than shader overlay

Scenario: clear native overlay is lighter than regular native overlay
Test: `cargo test -p makepad-example-aichat aichat_native_clear_overlay -- --nocapture`
Given the startup substrate is `macos-native-clear`
When aichat applies native appearance
Then the extra Makepad overlay is reduced below regular native mode

Scenario: shader fallback remains non-native
Test: `cargo test -p makepad-example-aichat aichat_startup_glass_resolution -- --nocapture`
Given no macOS native substrate is requested
When startup glass appearance is resolved
Then the shader-only substrate remains the default fallback

Scenario: repeated native batches do not reinstall every frame
Test: `cargo test -p makepad-platform native_glass -- --nocapture`
Given the macOS platform receives an identical native glass batch repeatedly
When the second batch is installed
Then the cached batch result can be returned without another AppKit rebuild

Scenario: affected crates still compile
Test: `cargo check -p makepad-platform && cargo check -p makepad-example-aichat`
Given the tuning and cache changes are applied
When the affected crates are checked
Then they compile without errors
