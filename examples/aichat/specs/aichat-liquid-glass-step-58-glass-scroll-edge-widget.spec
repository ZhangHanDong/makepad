spec: task
name: "AI Chat Liquid Glass Step 58 Glass Scroll Edge Widget"
tags: [makepad, liquid-glass, scroll-edge, widgets, phase-h]
---

## Intent

Add a Makepad-rendered scroll edge semantic hook without creating native panels.
This is the first implementation step after the scroll edge policy: a reusable
visual widget exists, but it is not yet wired to scroll state in aichat.

## Decisions

- Implement scroll edge treatment as Makepad-rendered widgets first.
- Keep scroll edges visual-only and input-passthrough.
- Do not export scroll edges as native glass descriptors.

## Boundaries

### Allowed Changes

- `widgets/src/glass_panel.rs`
- `examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit aichat layout yet.
- Do not add native panels.
- Do not require scroll state plumbing.

## Acceptance Criteria

### Scenario: scroll edge widgets exist
Given scroll edge treatment starts as Makepad semantic styling
When the widget source is inspected
Then `GlassScrollEdge` and `GlassScrollEdgeBottom` are defined
Test: `rg "GlassScrollEdge|GlassScrollEdgeBottom" widgets/src/glass_panel.rs`

### Scenario: scroll edge widgets are Makepad-rendered only
Given scroll edge widgets must not create native descriptors
When the widget source is inspected
Then the scroll edge widget definitions do not set `native: true`
Test: `rg "GlassScrollEdge" widgets/src/glass_panel.rs && ! rg "GlassScrollEdge[^{]*\\{[^}]*native: true" widgets/src/glass_panel.rs`

### Scenario: widgets compile
Given the scroll edge widget style is part of makepad-widgets
When the widgets crate is checked
Then it compiles
Test: `cargo check -p makepad-widgets --release`

### Scenario: scroll edge implementation remains staged
Given aichat is not wired to scroll state yet
When policy and audit docs are inspected
Then they state the widget exists but aichat scroll-state wiring remains future work
Test: `rg "GlassScrollEdge|scroll-state wiring|not yet wired" examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
