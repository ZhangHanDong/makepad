# Plan: aichat liquid glass v3 macOS native

Supersedes the macOS-native parts of [aichat-liquid-glass-v3.plan.md](aichat-liquid-glass-v3.plan.md).

This plan is only for the macOS native substrate path based on `NSGlassEffectView`. It must stay independent from the cross-platform ShaderBackdrop blur/refraction plan.

## Current State

The `aichat-liquid-glass-phase1` branch has:

- proof native view insertion under the Makepad render view
- transparent Metal layer / render view cooperation
- runtime `NSGlassEffectView` class lookup
- `WindowNativeSubstrateResolvedEvent` platform-to-app result transport
- aichat fallback that stays ShaderOnly unless the platform reports `Installed`
- Makepad-only inactive dimming through existing focus events

On macOS versions without `NSGlassEffectView`, expected behavior is:

```text
[liquid-glass] state=1 reason=class-missing detail=NSGlassEffectView
[liquid-glass] app-substrate=shader state=ClassMissing reason=class-missing
```

## Goal

Finish and harden the native system material path when a target macOS runtime provides `NSGlassEffectView`.

Non-goals:

- cross-platform backdrop blur
- offscreen blur passes
- desktop capture
- per-panel native views
- runtime substrate switching

## Phase MN-1: Validate on Supported macOS

Run `makepad-example-aichat-macos-native-auto` on a macOS runtime that exposes `NSGlassEffectView`.
Use `makepad-example-aichat-macos-native` and `makepad-example-aichat-macos-native-clear` for explicit Regular/Clear style comparison after the auto smoke test passes.

Expected success logs:

```text
[liquid-glass] state=4 substrate=macos-native style=regular style_raw=0
[liquid-glass] app-substrate=macos-native state=Installed reason=installed-on-proofed-hierarchy
```

Acceptance:

- aichat starts without crash
- native glass is visibly behind Makepad content
- Makepad text, Markdown, input, buttons, and Splash content remain interactive
- `Glass` slider still controls overlay readability
- `AICHAT_GLASS_BACKEND=shader` never creates native glass

## Phase MN-2: Confirm Style Mapping

The current runtime implementation uses integer style values:

```text
Regular -> 0
Clear   -> 1
```

For validation only, these raw values can be overridden without rebuilding:

```text
AICHAT_MACOS_GLASS_STYLE_REGULAR_RAW=<NSInteger>
AICHAT_MACOS_GLASS_STYLE_CLEAR_RAW=<NSInteger>
```

If either override is set, the platform logs a `style-override` line before applying `setStyle:`. Invalid override values are ignored and logged as `style-override-invalid`.

Before calling this production-ready, verify these values against a macOS SDK/runtime that exposes `NSGlassEffectView.Style`.

Preferred verification:

```swift
import AppKit
print(NSGlassEffectView.Style.regular.rawValue)
print(NSGlassEffectView.Style.clear.rawValue)
```

Acceptance:

- style integers are documented with SDK/runtime source
- mismatch is fixed before public release

## Phase MN-3: Preflight Completeness

Keep all Objective-C interaction preflighted. Do not rely on catching ObjC exceptions from Rust.

Required checks:

- class exists
- class can initialize with frame
- `alloc/init` returns non-nil
- instance responds to `setStyle:`
- hierarchy attach succeeds
- optional selectors (`setTintColor:`, `setCornerRadius:`) are checked before use

Acceptance:

- failure states map to the existing state machine
- no unsupported selector is sent without a responds-to check
- logs keep the stable `[liquid-glass] state=N reason=...` form

## Phase MN-4: Visual Tuning

Tune `NativeOverlay` values only after native glass is visible on a supported system.

Focus areas:

- reduce double highlight if native glass already provides edge light
- keep Markdown code blocks readable
- ensure generated Splash roots do not cover the native substrate
- compare `Regular` vs `Clear`

Acceptance:

- bright wallpaper text remains readable
- dark wallpaper still shows material
- no "low alpha without glass" state is possible

## Phase MN-5: Known v1 Boundaries

Do not solve these in the first native release unless they block basic visibility:

- runtime switching
- fullscreen-specific glass behavior
- Stage Manager artifacts
- multi-display material quirks
- rounded window shape sync
- AppKit inactive overlay

The current Makepad-only inactive dimming is the v1 inactive strategy.

## Relationship to ShaderBackdrop

macOS native owns:

- native AppKit substrate
- platform view hierarchy
- substrate result events

ShaderBackdrop owns:

- offscreen Makepad scene texture
- blur passes
- refraction/displacement shader work

Do not mix these into one implementation PR. They can share high-level `GlassAppearance` and `GlassPanelPreset` concepts, but they have different failure modes and validation paths.
