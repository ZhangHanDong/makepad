# aichat Liquid Glass v1 Release Notes

## Scope

This release adds the first macOS native Liquid Glass path for aichat while keeping the existing shader glass path as the default cross-platform behavior.

Supported startup backends:

```text
unset                -> shader glass
shader               -> shader glass
macos-native         -> native macOS glass, regular style, if available
macos-native-clear   -> native macOS glass, clear style, if available
auto                 -> native regular style if available, otherwise shader glass
```

Native availability is detected at runtime through Objective-C class lookup for `NSGlassEffectView`. Unsupported runtimes fall back to shader glass without crashing.

## Verified Native Path

On the current macOS 26 validation runtime, the native path reaches State 4:

```text
[liquid-glass] state=4 substrate=macos-native style=regular style_raw=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=macos-native state=Installed reason=installed-on-proofed-hierarchy
```

Style mapping confirmed on this runtime:

```text
NSGlassEffectView.Style.regular.rawValue == 0
NSGlassEffectView.Style.clear.rawValue   == 1
```

State 4 means the native substrate was installed on the proofed window hierarchy and the app accepted the platform result as its source of truth. It does not mean per-frame AppKit layer visibility is continuously re-verified.

## Visual Tuning

Native mode uses a lower Makepad overlay preset than shader mode so the AppKit substrate remains visible:

- lower shell/main tint
- reduced highlight and border strength
- reduced shader noise
- halo disabled
- brighter foreground text for bright and dark wallpaper readability

The `Glass` slider still adjusts Makepad overlay readability. It does not change the AppKit native material itself.

## Splash Guard

AI-generated Splash blocks can otherwise cover the native substrate with full-size opaque roots. In native mode, aichat now guards obvious generated roots:

- fenced `splash` blocks only
- root `View`, `RoundedView`, or `SolidView`
- `width: Fill` and `height: Fill`
- opaque `draw_bg.color` or `draw_bg +: { color: ... }`

Detected opaque root colors are clamped to `0x80` alpha and logged:

```text
[liquid-glass] splash-opaque-root view=<View|RoundedView|SolidView> action=clamp-alpha
```

The guard is intentionally narrow. It is a v1 degradation path, not a full Splash static analyzer.

## Known v1 Limitations

The first native release intentionally does not handle:

- runtime substrate switching
- native fullscreen-specific behavior
- Stage Manager-specific artifacts
- multi-display material transitions
- rounded window shape synchronization between Makepad and AppKit
- AppKit-native inactive-window overlay
- cross-platform real blur/refraction

The current inactive behavior is Makepad-only dimming. ShaderBackdrop blur/refraction is a separate future phase requiring offscreen scene capture, blur passes, and `GlassPanel` sampling.

## Validation

Use Makepad Studio remote release runs for UI validation. Do not use raw `cargo run` as the runtime UI validation path.

Recommended smoke targets:

```text
makepad-example-aichat-macos-native
makepad-example-aichat-macos-native-clear
```

Required smoke checks:

- app starts without crash
- State 4 native logs appear on supported macOS
- unsupported macOS falls back to shader logs without crash
- text, Markdown, input, buttons, and Splash output remain interactive
- native glass remains visible behind the Makepad content by direct visual inspection
