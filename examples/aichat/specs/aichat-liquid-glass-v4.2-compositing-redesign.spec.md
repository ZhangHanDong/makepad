# aichat Liquid Glass v4.2 Compositing Redesign

## Status

Design route. No implementation is implied by this document.

v4.2 follows the v4.1 native proof work. v4.1 proved that Makepad can create
Apple native glass views and align them with aichat panels. It did not prove
complete Liquid Glass. The striped proof target showed the native underlay was
visible and resizable, but the user could not visually distinguish meaningful
interior glass treatment. The current v4.1 model places native glass below the
Makepad Metal layer, so AppKit glass cannot act on the Makepad-rendered content
above it.

Related documents:

- [aichat-liquid-glass-v4-apple-native-full.spec.md](aichat-liquid-glass-v4-apple-native-full.spec.md)
- [aichat-liquid-glass-v4.1-release-notes.md](aichat-liquid-glass-v4.1-release-notes.md)
- [aichat-liquid-glass-v4.1-manual-visual-checklist.md](aichat-liquid-glass-v4.1-manual-visual-checklist.md)
- [aichat-liquid-glass-step-12-native-compositing-proof.spec](aichat-liquid-glass-step-12-native-compositing-proof.spec)
- [aichat-liquid-glass-step-13-proof-conclusion-and-log-throttle.spec](aichat-liquid-glass-step-13-proof-conclusion-and-log-throttle.spec)

## Objective

Find and prove a compositing model that can deliver recognizable interior Liquid
Glass while preserving Makepad's core requirements:

- Makepad owns text, Markdown, input, generated Splash UI, and cross-platform
  fallback rendering.
- Apple platforms can use native Liquid Glass APIs where those APIs can actually
  contribute visible material behavior.
- Unsupported platforms stay on shader fallback.
- Input remains deterministic and inspectable through Makepad Studio.

## v4.1 Finding

The v4.1 hierarchy is:

```text
NSWindow content root
    native proof/substrate/glass views
    transparent Makepad Metal view
        all Makepad content
```

This is good for:

- proving native class/selector availability
- verifying native view insertion and resize
- drawing edge/radius native glass under Makepad content
- keeping all input routed through Makepad

It is not enough for complete Liquid Glass because the native glass view is not
compositing Makepad's Metal content. Apple glass samples and transforms what is
behind the native view in the AppKit/UIKit hierarchy; Makepad content is above
it.

## Design Requirement

v4.2 must prove, with a minimal prototype, at least one of these is true:

1. Apple native glass can visibly treat Makepad-rendered content if Makepad
   content is moved into an appropriate native-hosted compositing position.
2. Apple native glass cannot treat Makepad Metal content in the way aichat
   needs, so complete Liquid Glass must use `ShaderBackdrop` for interiors and
   reserve Apple native APIs for window/substrate/control affordances.

Until one of those is proven, aichat should not claim full native Liquid Glass.

## Candidate Architectures

### A. v4.1 Under-Metal Native Panels

```text
native glass panels
transparent Metal view with all Makepad content
```

Status: rejected for full Liquid Glass.

Use only for native underlay/substrate proof and limited edge-visible material.
The Phase 12 proof showed striped underlay visibility but no clearly recognizable
interior glass.

### B. Native Glass Above Metal

```text
Metal view with all Makepad content
native glass panels above Metal
```

Status: not viable for v4.2 as a general app model.

Problems:

- Native panels cover Makepad-rendered text and controls.
- AppKit hit testing becomes the owner unless every panel is explicitly
  passthrough and visually transparent to interaction.
- Makepad content still is not inside the native glass surface; it is simply
  behind or covered by it.

This may be useful only as a diagnostic prototype to confirm what AppKit samples
when glass is above a Metal layer.

### C. Multi-Metal Layer Interleaving

```text
Metal layer for content behind glass
native glass panel
Metal layer for content above glass
```

Status: possible but high risk.

This is the closest native-API route for full glass if AppKit can sample the
lower Metal layer through native glass. It requires splitting Makepad rendering
into at least two compositing passes/layers:

- behind-glass scene content
- foreground text/controls

Risks:

- Multiple Metal layers per window complicate draw ordering, input mapping,
  Studio screenshots, resize, DPI, focus, and damage tracking.
- It may still fail if AppKit cannot use the Metal layer as useful glass input.
- Scrolling content and generated UI would need explicit layer assignment.

### D. Native-Hosted Glass Islands

```text
native glass host view
    native glass surface
    transparent Makepad island layer/view for foreground content
main Makepad Metal view for the rest of the app
```

Status: research candidate.

Each glass island owns a native host. Makepad renders panel-local foreground
content into a separate transparent surface layered with the native glass view.

Risks:

- Requires render-surface extraction for Makepad subtrees.
- Multiple small Metal views/layers can be expensive.
- Studio widget queries and screenshots need a cross-surface story.
- Input forwarding must map island coordinates back to the Makepad widget tree.

### E. ShaderBackdrop Interior + Apple Native Window/Control Affordances

```text
single Makepad Metal view
    Makepad-rendered backdrop blur/refraction for glass interiors
optional Apple native substrate/control affordances outside the sampled content
```

Status: likely practical fallback if full native sampling cannot be proven.

This gives Makepad full control over interior refraction, blur, and readability.
Apple native APIs can still be used where they are naturally useful, such as
window material, native controls, or platform-specific chrome.

This is not "full native Liquid Glass," but it may be the best complete visual
result for Makepad's renderer.

## Required Prototypes

v4.2 implementation must start with standalone prototypes before changing
aichat production UI:

1. **Above-Metal Sampling Probe**
   - One Metal layer draws a moving high-contrast pattern.
   - One `NSGlassEffectView` is placed above it.
   - Result determines whether AppKit glass visibly samples Metal content.

2. **Two-Layer Interleave Probe**
   - Lower Metal layer draws patterned background.
   - Native glass view sits between lower and upper Metal layers.
   - Upper Metal layer draws text/controls.
   - Result determines whether full native glass can be achieved with split
     Makepad rendering.

3. **Input Probe**
   - Same hierarchy as the winning visual probe.
   - Verify clicks, text input, scroll, and window drag can still route through
     Makepad without AppKit stealing events.

4. **Studio Probe**
   - Verify Studio screenshot, widget dump, and run lifecycle still work.
   - If screenshots capture only one Metal layer, document that limitation.

## Decision Gate

After prototypes, choose one route:

- **Route NativeInterleave:** if two-layer native sampling works and input/Studio
  risks are bounded.
- **Route ShaderBackdropInterior:** if native sampling does not work or the
  renderer split is too expensive for aichat.
- **Route NativeUnderlayOnly:** if v4.1 remains useful only as a platform proof
  and not as a user-facing full glass mode.

The chosen route must update `GlassBackend` terminology so user-facing names do
not overclaim. For example:

```rust
pub enum GlassBackend {
    ShaderOverlay,
    AppleNativeUnderlay,
    AppleNativeInterleave,
    ShaderBackdrop,
}
```

Names are illustrative; final naming should match implementation evidence.

## Non-Goals

- Do not implement v4.2 directly inside aichat before the probes are complete.
- Do not add arbitrary native panels above the main Makepad view as a quick fix.
- Do not move scrolling list rows into native views.
- Do not make native interactive controls part of the first compositing probe.
- Do not claim full native Liquid Glass based only on State 4 logs.

## Acceptance

v4.2 design is ready for implementation when:

- the four required prototypes are represented as tasks with concrete files and
  verification commands
- the prototype outputs can distinguish native sampling from flat underlay
- the decision gate has explicit pass/fail criteria
- v4.1 docs link to this redesign as the next phase
- no user-facing target is renamed to "full native" before visual evidence exists
