# aichat Liquid Glass v4.1 Release Notes

## Scope

v4.1 adds the first multi-panel Apple-native Liquid Glass path for aichat on
macOS. It keeps the existing shader overlay path as the default cross-platform
behavior and only enables native panel descriptors when the resolved substrate
is `GlassSubstrate::MacosNative`.

The active v4.1 native surface model is:

- one `GlassContainer` per window
- up to twelve visible native panels per window
- native panels below the Makepad Metal layer
- all Makepad controls and text rendered above native glass
- passthrough hit testing only
- rounded-rect and capsule descriptor shapes
- sRGB tint conversion for native tint colors

## Runtime State Logs

The v1 compatibility log remains a single per-window summary line. In a
multi-panel run it reports the container-level aggregate state; in a single
panel run it keeps the old meaning.

```text
[liquid-glass] state=4 substrate=macos-native style=regular style_raw=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

State 4 means the platform accepted the native substrate path and aichat should
treat macOS native glass as active. It does not prove that every native panel is
visually strong enough on every wallpaper, nor does it continuously re-verify
AppKit layer visibility after every frame.

v4.1 also emits batch/container/panel diagnostics:

```text
[liquid-glass] native-container state=installed platform=macos containers=1 panels_installed=M panels_failed=F
[liquid-glass] native-panel id=<id> state=<state> reason=<reason>
```

Use the v4.1 lines to debug individual panel failures. Use the v1 compatibility
line only for older aichat state handling and quick startup smoke checks.

## Validation Status

A Studio release run is required for visual UI validation. `cargo check` and
unit tests validate the Rust wiring, but they do not prove native glass is
visible in the rendered app.

Current status:

- compile and unit-test validation: available
- macOS 26 visual validation: first Studio release pass recorded on 2026-05-02
- current documentation state: native regular and clear styles reached State 4
  with four installed panels and no failed panels

The first manual pass used Makepad Studio RunItem validation for both required
targets. `macos-native` reached `style=regular style_raw=0`; `macos-native-clear`
reached `style=clear style_raw=1`. Both reported
`containers=1 panels_installed=4 panels_failed=0`, and the user confirmed the
native glass was visible. Step 11 reduces the Makepad overlay contribution so
the native material is less hidden by app-rendered tint, highlight, noise, and
halo.

The Step 11 Studio rerun confirmed regular and clear native targets still reach
State 4 after visual tuning. It also confirmed identical native glass batches no
longer produce repeated install logs after startup.

Step 12/13 added a striped native proof substrate and a transparent-overlay
proof mode. The striped proof became visible and covered the window after
resize, which confirms the native underlay is present and correctly layered.
However, the user could not visually identify meaningful interior Liquid Glass
treatment beyond the flat striped underlay. v4.1 should therefore be described
as an Apple-native underlay/substrate proof, not complete Liquid Glass. Complete
Liquid Glass requires a later compositing design that is not limited to native
panels below the Makepad Metal layer.

Next phase:

- [aichat-liquid-glass-v4.2-compositing-redesign.spec.md](aichat-liquid-glass-v4.2-compositing-redesign.spec.md)
- [aichat-liquid-glass-v4.2-compositing-redesign.plan.md](aichat-liquid-glass-v4.2-compositing-redesign.plan.md)

The v4.2 compositing redesign starts with native/Metal sampling probes before
any aichat production UI integration.

## Known v4.1 Limitations

v4.1 intentionally does not handle:

- runtime switching between shader and native substrates after startup
- fullscreen-specific native glass behavior
- Stage Manager-specific window/material artifacts
- multiple displays and display-to-display material transitions
- rounded-corner synchronization between Makepad window shape and native AppKit
  views
- interactive native glass controls
- native popup/modal windows
- iOS/iPadOS native backend
- native + shader backdrop mixing in one window
- complete interior Liquid Glass treatment in the current under-Metal
  compositing model

ShaderBackdrop remains a separate future phase. It requires offscreen scene
capture, blur/refraction passes, and panel sampling; it should not be mixed with
the AppleNative backend in the same v4.1 window.

## Required Smoke Targets

Use Makepad Studio remote release runs for UI validation:

```text
makepad-example-aichat-macos-native
makepad-example-aichat-macos-native-clear
```

Minimum visual checks:

- supported macOS 26 runtime reaches State 4
- native container log reports `containers=1`
- native panel log reports expected installed panel count
- text remains readable over bright and dark wallpapers
- shell, sidebar, main area, and composer show native glass under Makepad content
- generated Splash content does not cover the native substrate with opaque roots
