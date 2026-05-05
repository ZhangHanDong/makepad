spec: task
name: "AI Chat Liquid Glass Step 17 Input And Studio Probe"
tags: [makepad, aichat, liquid-glass, macos-native, studio-validation, input]
---

## Intent

Record the input and Studio implications of the v4.2 native compositing probes.
The above-Metal native glass overlay is useful evidence for visual sampling,
but it is not a production hierarchy because it lives above Makepad-rendered
widgets and is absent from Studio framebuffer screenshots.

## Decisions

- Keep Makepad as the input owner for aichat v4.x production targets.
- Treat native glass above the main Makepad view as a diagnostic overlay only.
- Studio `Screenshot` captures the Makepad framebuffer; system screenshots are
  required to see AppKit overlays above the framebuffer.
- `WidgetTreeDump` remains useful for Makepad widgets, but it cannot prove
  native AppKit overlay visibility.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-17-input-studio-probe.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-above-metal-probe-result.md`

### Forbidden

- Do not make `NSGlassEffectView` an interactive aichat control host in v4.2.
- Do not rely on Studio framebuffer screenshots to prove native overlay
  visibility.
- Do not claim the diagnostic overlay preserves production input semantics.

### Out of Scope

- Native input forwarding.
- Multi-window popup/modal hit testing.
- Accessibility behavior for native overlay controls.

## Acceptance Criteria

Scenario: Studio screenshot limitation is documented
Test: `rg "Studio framebuffer screenshot|system screenshot" examples/aichat/specs/aichat-liquid-glass-v4.2-above-metal-probe-result.md examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given a native AppKit overlay is above the Makepad Metal view
When Studio captures the app framebuffer
Then the native overlay is documented as absent from the Studio framebuffer capture

Scenario: input ownership decision remains Makepad
Test: `rg "Keep Makepad as the input owner|native glass above the main Makepad view as a diagnostic overlay only" examples/aichat/specs/aichat-liquid-glass-step-17-input-studio-probe.spec`
Given aichat has text inputs, buttons, scroll, and generated UI
When selecting the v4.2 route
Then input remains Makepad-owned for production targets

Scenario: route decision includes input evidence
Test: `rg "Input evidence|diagnostic overlay only" examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given the above-Metal overlay can cover Makepad content
When closing v4.2
Then the decision explains why it is not a production input hierarchy

Scenario: native overlay is not accepted as input proof
Test: `rg "Do not claim the diagnostic overlay preserves production input semantics|not a production input hierarchy" examples/aichat/specs/aichat-liquid-glass-step-17-input-studio-probe.spec examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given an above-Metal native overlay visually works
When it lacks a Makepad-owned hit-test proof for text input, scroll, and drag
Then v4.2 rejects it as production input evidence
