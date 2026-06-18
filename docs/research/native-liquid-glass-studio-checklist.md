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
{"Click":{"build_id":[BUILD_ID],"x":[RESIZE_PROBE_X],"y":[RESIZE_PROBE_Y]}}
{"WidgetTreeDump":{"build_id":[BUILD_ID]}}
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

The `Resize Probe` button cycles through large / small / default validation
sizes (`1120x780`, `760x520`, `820x560`) and should emit a request line with a
target label plus a geometry-change validation summary:

```text
[liquid-glass] standalone-native-example resize-probe=request current=820x560 target=1120x780 target_label=large
[liquid-glass] standalone-native-example resize-validation source=window-geom-change mode=Panels size=1120x780 panels=3 panels_fit=true controls=0 controls_fit=true spacing=28 dpi=2.00 scale_changed=false
```

Verbose native container/panel update logs are off by default. Set
`MAKEPAD_NATIVE_GLASS_DIAGNOSTIC_LOGS=1` when collecting logs such as
`native-container-spacing`, `native-panel-update`, or
`native-container-panel-parent`.

## Visual Acceptance

Use direct visual inspection or a normal macOS screenshot for native material.
Studio `Screenshot` captures the Metal framebuffer and is still useful for
foreground UI layout, but it does not capture the AppKit native underlay.

Accept the run only when:

- The window is transparent and borderless.
- Native glass is visible below the Makepad foreground layer.
- No opaque root background covers the native underlay.
- The three native panels track window resize.
- `Resize Probe` cycles through small / default / large validation sizes and
  logs `panels_fit=true`; in Controls mode it also logs
  `controls=5 controls_fit=true`.
- Resize validation summaries include `dpi=... scale_changed=...` so
  backing-scale changes are visible in logs when the window moves across
  displays.
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
