# AppleNativeInterleave Renderer Requirements

## Status

Research requirements for a future Apple-native interleave backend. This
document does not implement `AppleNativeInterleave`.

## Current Renderer Evidence

The current macOS renderer creates the primary window Metal layer through
`CAMetalLayer` in `platform/src/os/apple/macos/macos.rs`.

Window draw passes are rendered through `DrawPassMode::MTKView`,
`DrawPassMode::Drawable`, or `DrawPassMode::Resizing` in the macOS event loop.
Offscreen draw passes use `DrawPassMode::Texture` in `platform/src/os/apple/metal.rs`.

This means the current production hierarchy is effectively:

```text
NSWindow content root
    Apple native underlay / substrate views
    one Makepad Metal window surface
        all Makepad text, controls, Markdown, generated UI, and overlays
```

The landed `AppleNativeUnderlay` path inserts native glass below the primary
Makepad Metal surface. That is enough to prove class availability, view
insertion, resize alignment, and State 4 installation. It is not enough for full
native interior Liquid Glass because Makepad foreground content is above the
native glass view.

## Current macOS Renderer Entrypoints

The first `AppleNativeInterleave` prototype must change the renderer at the
native surface boundary, not only in aichat widget code. The current inspected
entrypoints are:

| Area | Current entrypoint | Interleave implication |
| --- | --- | --- |
| Surface creation | `platform/src/os/apple/macos/macos.rs::MetalWindow::new` creates one `CAMetalLayer`, configures it, and assigns it to the view with `setLayer: ca_layer`. | A real interleave needs at least lower and upper Makepad-presented native surfaces instead of a single primary layer. |
| Popup surface creation | `platform/src/os/apple/macos/macos.rs::MetalWindow::new_popup` repeats the same one-`CAMetalLayer` setup for popup windows. | Popup/modal native glass remains a separate phase; the first interleave prototype should stay on the main window path. |
| Window repaint | `platform/src/os/apple/macos/macos.rs::Cx::handle_repaint` resizes `metal_window.ca_layer`, obtains `nextDrawable`, and renders the window pass with `DrawPassMode::Drawable` or `DrawPassMode::Resizing`. | The split prototype needs a way to decide which window pass draws to the lower surface and which draws to the upper surface. |
| Offscreen passes | Child/standalone draw passes use `DrawPassMode::Texture`. | This remains useful for shader backdrop work, but it is not hosted as a native layer behind Apple glass and therefore does not prove native interleave. |
| Render pass and screenshot | `platform/src/os/apple/metal.rs::Cx::draw_pass` binds either an `MTKView` render pass descriptor, a drawable texture, or offscreen textures. `build_screenshot_struct` currently copies from the presented window texture for `DrawPassMode::Drawable` / `MTKView`. | Studio screenshot behavior must be explicitly defined once there are lower and upper presented surfaces; capturing only one presented window texture may no longer represent the full real composition. |

The first implementation slice should therefore introduce an internal renderer
surface model before exposing the backend. A useful minimal shape is:

```text
MacosMetalSurfaceRole::Primary     // current behavior
MacosMetalSurfaceRole::LowerScene  // future native-glass sampling source
MacosMetalSurfaceRole::UpperUi     // future Makepad foreground/input surface
```

The initial landed code for that model should preserve `Primary` behavior until
the lower/upper surface creation and draw routing are implemented and visually
validated.

## Required Interleave Shape

A real `AppleNativeInterleave` backend needs a native hierarchy closer to:

```text
NSWindow content root
    lower Makepad native surface / Metal layer
        scene content behind glass
    Apple native glass container/view
        native Liquid Glass samples lower content
    upper Makepad native surface / Metal layer
        text, controls, generated UI, Studio-inspectable foreground
```

Offscreen texture rendering alone is not native interleave. In other words,
offscreen texture rendering alone is not native interleave. A
`DrawPassMode::Texture` pass can produce pixels, but Apple native glass cannot
sample a Makepad texture unless those pixels are presented through a native
surface/layer in the AppKit/UIKit hierarchy behind the glass view.

## Implementation Gates

### Visual Gate

Visual gate.

- Native glass must visibly sample the lower Makepad-rendered native surface.
- The proof must use Makepad-rendered lower content, not a native-only label,
  static bitmap, or Core Animation placeholder.
- The upper Makepad foreground must remain readable and visibly above the glass.

### Input Gate

Input gate.

- Makepad remains the input owner for text input, scroll, clicks, window drag,
  command menus, and generated UI.
- Native glass views remain passthrough unless a later native-control phase
  defines event forwarding.

### Studio Gate

Studio gate.

- Studio `Screenshot` behavior must be defined for multiple Makepad surfaces.
- `WidgetTreeDump` and `WidgetQuery` must still address the foreground widget
  tree predictably.
- If Studio can only capture one surface, that limitation must be documented
  before the backend is exposed.

### Resize/DPI Gate

Resize/DPI gate.

- Lower surface, native glass views, and upper surface must stay aligned during
  window resize, scale-factor changes, and display moves.
- Logical Makepad coordinates must still convert at the platform boundary.

### Lifecycle Gate

Lifecycle gate.

- Startup, shutdown, `ClearBuild`, and native view cleanup must destroy all
  layers/views without leaking stale foreground or glass surfaces.
- The backend must define what happens when native class/selector lookup fails:
  full fallback to shader, not partial interleave.

## Non-Evidence

The following do not prove `AppleNativeInterleave`:

- State 4 logs from `AppleNativeUnderlay`.
- A native glass view above the single Makepad Metal surface.
- Studio framebuffer screenshots that omit native AppKit overlays.
- Offscreen `DrawPassMode::Texture` output that is never hosted as a native
  layer behind Apple glass.
- Native-only text or controls placed above glass.

## Current Decision

`AppleNativeInterleave` now has a guarded production-preview backend behind:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_ENABLE_APPLE_NATIVE_INTERLEAVE=production-preview
```

The preview has concrete layer-order evidence and reaches State 4, but it
should not become the default or a validated production backend until the
manual system-composited visual gate passes with the production lower scene.
