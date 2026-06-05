# Native Liquid Glass Studio Checklist

Use this checklist for `makepad-example-native-liquid-glass` runtime validation.
All UI runs must go through Makepad Studio remote control.

## Remote Flow

Start one persistent bridge process:

```text
target/release/cargo-makepad studio --studio=127.0.0.1:8001
```

If Studio is using the fallback port, use `127.0.0.1:8002`.

Launch sequence:

```json
{"ListBuilds":[]}
{"ClearBuild":{"build_id":[OLD_ID]}}
{"RunItem":{"mount":"makepad","name":"makepad-example-native-liquid-glass"}}
```

Validation sequence after `BuildStarted`:

```json
{"WidgetTreeDump":{"build_id":[BUILD_ID]}}
{"Screenshot":{"build_id":[BUILD_ID],"kind_id":0}}
{"Click":{"build_id":[BUILD_ID],"x":493,"y":228}}
{"Screenshot":{"build_id":[BUILD_ID],"kind_id":0}}
```

Use the current widget dump for click coordinates. Do not apply extra DPI math.

## Required Logs

The fresh run must reach native state 4:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=3 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

Mode and toggle clicks should emit status lines like:

```text
[liquid-glass] standalone-native-example tuning Panels  Clear  tint 22%  spacing 28  radius 44  makepad-glint:off
[liquid-glass] standalone-native-example tuning Panels  Clear  tint 22%  spacing 28  radius 44  makepad-glint:on
```

## Visual Acceptance

Use direct visual inspection or a normal macOS screenshot for native material.
Studio `Screenshot` captures the Metal framebuffer and is still useful for
foreground UI layout, but it does not capture the AppKit native underlay.

Accept the run only when:

- The window is transparent and borderless.
- Native glass is visible below the Makepad foreground layer.
- No opaque root background covers the native underlay.
- The three native panels track window resize.
- `Clear` and `Regular` styles are visibly different on macOS 26.
- `Geometry` shows rounded-rect and capsule descriptor mapping.
- `Container` changes native `setSpacing:` through Near / Threshold / Far.
- `Debug` exposes mode, style, tone, panel count, z order, hit-test policy, and
  `makepad-glint:on/off` state.
- `Readability` exposes Mixed / Bright / Dark text-token probes.
- `Glint Off` hides the Makepad demo overlay so Apple native glass can be
  inspected alone.

## Non-Acceptance

Do not treat these as native glass proof:

- The gold `GoldGlintEdge` animation. It is a Makepad shader overlay.
- Studio framebuffer screenshots alone. They do not include the AppKit underlay.
- `cargo test` or `cargo build` without a fresh Studio `RunItem`.
