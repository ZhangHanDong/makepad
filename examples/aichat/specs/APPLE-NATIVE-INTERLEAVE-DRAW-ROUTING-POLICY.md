# AppleNativeInterleave Draw Routing Policy

## Status

This document defines the first aichat routing policy for a future
`AppleNativeInterleave` backend. The lower/upper draw routing is not implemented yet.

## Three-Layer Stack

The intended production stack is:

```text
LowerScene
    Makepad-rendered non-interactive scene/background/proof content

Apple native glass
    NSGlassEffectContainerView / NSGlassEffectView panels
    passthrough, not input-owning in this phase

UpperUi
    Makepad-rendered text, input, controls, Markdown, generated UI,
    hit-testable widgets, focus state, selection, cursor affordances
```

`LowerScene` exists so Apple native glass can sample a real native-presented
Makepad surface. It is not a replacement for aichat's foreground UI.

## LowerScene Policy

`LowerScene` starts with non-interactive scene/background/proof content only.
It may contain:

- window background scene content
- diagnostic moving-pattern or color proof content
- future non-interactive ambient content intended to be sampled through glass

`LowerScene` must not receive input, focus, text editing, scroll ownership, menu
commands, selection state, or generated UI actions. It must not contain
foreground chat text, composer text, toolbar controls, model pickers, Markdown
content, or any hit-testable widgets.

## UpperUi Policy

`UpperUi` keeps all user-facing and input-owning aichat UI:

- text and labels
- text input and composer state
- buttons, controls, toolbar affordances, and model/backend controls
- Markdown and rendered generated UI
- chat messages and hit-testable widgets
- focus rings, cursors, drag regions, selection, and accessibility-facing UI

This keeps Makepad-owned input predictable while native glass remains a visual
middle layer.

## Native Glass Policy

Apple native glass sits between `LowerScene` and `UpperUi`. In this phase it
remains passthrough and must not own input. Native interactive controls remain
outside this backend until a separate event-forwarding policy is implemented.

The first routed prototype should therefore prove this order:

```text
lower Makepad native surface -> Apple native glass -> upper Makepad foreground
```

It must not fake success with native-only labels, static Core Animation
placeholders, or offscreen textures that are not hosted as native layers.

## Exposure Gates

`AppleNativeInterleave` must remain reserved until all of the following are
proven with concrete artifacts:

- Visual gate: native glass visibly samples `LowerScene`, and `UpperUi` remains
  readable above it.
- Input gate: clicks, scroll, text input, focus, drag regions, and generated UI
  remain Makepad-owned.
- Studio screenshot gate: Studio capture behavior is documented for lower and
  upper surfaces, including any missing native AppKit composition.
- Resize/DPI gate: lower surface, native glass views, and upper surface stay
  aligned during resize and backing-scale changes.
- Lifecycle gate: startup, `ClearBuild`, shutdown, and fallback remove all
  native views/layers without stale hidden surfaces.

Until those gates are satisfied, `AICHAT_GLASS_BACKEND=apple-native-interleave`
must keep falling back rather than becoming an available backend.
