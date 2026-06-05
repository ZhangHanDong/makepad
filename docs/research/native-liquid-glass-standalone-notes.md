# Native Liquid Glass Standalone Notes

The standalone `makepad-example-native-liquid-glass` app is the canonical local
validation bench for Apple native Liquid Glass integration.

## Native Versus Demo Layers

The Apple native path is the `GlassContainer { native: true }` plus its
`GlassPanel { native: true }` descriptors. On macOS this should install
`NSGlassEffectContainerView` and `NSGlassEffectView` instances below the Metal
foreground layer.

`GoldGlintEdge` is intentionally different. It is a Makepad-rendered shader
overlay used to make edge motion easier to see during manual demos. It is not an
Apple Liquid Glass API, and it must not be used as evidence that native glass is
working.

Use the in-app `Glint Off` / `Glint On` control when validating native behavior:

- `Glint Off`: inspect Apple native glass without Makepad shader decoration.
- `Glint On`: inspect the demo overlay and native descriptor pulse together.

The status readout includes `makepad-glint:on` or `makepad-glint:off` so logs and
screenshots preserve whether the overlay was enabled.

## v4.1 Descriptor Coverage

The standalone example currently exercises these Apple native descriptor fields:

| Field | Standalone coverage |
| --- | --- |
| `style` | `Clear` / `Regular` controls update `native_style`. |
| `tint` | `Tint -` / `Tint +` and `Tone` update sRGB `native_tint`. |
| `shape` | `Geometry` and the bottom pill use rounded rect plus capsule panels. |
| `radius` | `Radius -` / `Radius +` update rounded-rect `native_radius`. |
| `z_order` | Main, top, and bottom panels use z order `0 / 1 / 2`. |
| `hit_test` | All native panels explicitly set `native_hit_test` to passthrough. |
| `container spacing` | `Spacing -` / `Spacing +` and `Container` presets update `GlassContainer.spacing`. |

v4.1 still rejects `NativeGlassHitTest::Interactive` for panels. Native
interactive controls are validated separately through the Controls mode and the
button action/focus probe runnables.

## Visual Validation

Use Studio remote to launch the runnable, but remember that Studio `Screenshot`
captures the Metal framebuffer only. It is useful for verifying foreground UI
layout, widgets, and mode changes, but it does not capture the AppKit native
underlay. Use a normal macOS screenshot or direct visual inspection for the
native material.

Expected native evidence remains:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=3 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```
