spec: task
name: "AI Chat Liquid Glass Step 77 aichat iOS Native Substrate"
tags: [makepad, liquid-glass, apple-native, ios, aichat, phase-g]
---

## Intent

Teach aichat to recognize the iOS native glass style variants introduced by
Step 76. Without this, an iOS backend success event would still resolve to the
default shader appearance at the app layer.

## Decisions

- Add an `IosNative` substrate variant while preserving the existing
  `MacosNative` variant.
- Reuse the current regular/clear style enum for native overlay tuning.
- Treat both macOS and iOS native substrates as `NativeOverlay` for Makepad
  overlay intensity and descriptor export.
- Keep fullscreen fallback macOS-specific.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not claim iOS runtime validation is complete.
- Do not change platform backends.
- Do not change shader backdrop behavior.

## Acceptance Criteria

### Scenario: iOS native resolved event maps to native appearance
Given an installed iOS native clear substrate event
When aichat resolves the native substrate appearance
Then it returns `GlassSubstrate::IosNative { style: Clear }`
Test: `cargo test -p makepad-example-aichat aichat_ios_native_substrate_event_resolves_to_native_overlay --release`

### Scenario: non-installed iOS event still falls back
Given a non-installed iOS native substrate event
When aichat resolves the native substrate appearance
Then it returns the default shader appearance
Test: `cargo test -p makepad-example-aichat aichat_ios_native_substrate_event_requires_installed_state --release`

### Scenario: native overlay predicate includes iOS
Level: unit
Targets: `GlassSubstrate::is_native`, `GlassAppearance::panel_preset`
Given iOS native appearance is active
When overlay presets and native-panel export are computed
Then iOS native uses the same native overlay path as macOS native
Test: `cargo test -p makepad-example-aichat aichat_ios_native_uses_native_overlay_preset --release`

### Scenario: fullscreen fallback remains macOS-specific
Given iOS native appearance is active
When fullscreen fallback is evaluated
Then it does not enter the macOS fullscreen fallback path
Test: `cargo test -p makepad-example-aichat aichat_ios_native_does_not_enter_macos_fullscreen_fallback --release`

### Scenario: platform backends are untouched
Given this step only changes aichat event handling
When the staged diff is inspected
Then no platform backend files are changed
Test: `! git diff --name-only --cached | rg "^platform/src/"`

### Scenario: completion audit remains honest
Given aichat can recognize iOS native events
When the completion audit is inspected
Then it still says UIKit backend is not runtime-validated beyond the installer skeleton
Test: `rg "not runtime-validated beyond the dynamic installer skeleton" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: aichat compiles
Given aichat handles iOS native substrate events
When aichat is checked
Then release compile passes
Test: `cargo check -p makepad-example-aichat --release`
