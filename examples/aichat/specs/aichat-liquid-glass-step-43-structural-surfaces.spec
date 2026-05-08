spec: task
name: "AI Chat Liquid Glass Step 43 Structural Surfaces"
tags: [makepad, aichat, liquid-glass, glass-surfaces, phase-f]
---

## Intent

Close the remaining Phase F direct GlassPanel exceptions in aichat by naming the
two structural native panels as semantic glass surfaces. This is a naming and
semantic-layer step only; it must not change platform native glass behavior.

## Decisions

- Add `GlassShell` as the reusable semantic alias for the outer app shell.
- Add `GlassMainSurface` as the reusable semantic alias for the main content panel.
- Keep both surfaces Makepad-rendered panels that may opt into native
  descriptors.
- Keep native interactivity out of scope.

## Boundaries

### Allowed Changes

- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`
- `examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `examples/aichat/specs/aichat-liquid-glass-step-43-structural-surfaces.spec`

### Constraints

- Do not edit `platform/src`.
- Do not add new native backend behavior.
- Do not change `GlassContainer` collection semantics.
- Do not make message rows or utility controls native panels.
- Do not remove app-local size, padding, or native descriptor values unless the
  shared alias intentionally owns the same defaults.

## Acceptance Criteria

### Scenario: structural semantic widgets are registered
Given the shared glass widget module
When it is inspected
Then it registers both `GlassShell` and `GlassMainSurface` as `GlassPanel` aliases
Test: `rg "mod\\.widgets\\.GlassShell = mod\\.widgets\\.GlassPanel|mod\\.widgets\\.GlassMainSurface = mod\\.widgets\\.GlassPanel" widgets/src/glass_panel.rs`

### Scenario: aichat structural panels use semantic aliases
Given the aichat UI script
When it is inspected
Then `app_shell` uses `GlassShell` and `main_area` uses `GlassMainSurface`
Test: `rg "app_shell := GlassShell|main_area := GlassMainSurface" examples/aichat/src/main.rs`

### Scenario: Phase F audit is closed
Given the Phase F audit document
When it is inspected
Then it no longer says Phase F is partially landed or mostly implemented
And it documents the two structural surface aliases
Test: `rg "Phase F is landed|GlassShell|GlassMainSurface" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md && ! rg "partially landed|mostly implemented|GlassPanel Exceptions" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`

### Scenario: v4 status table reflects Phase F
Given the v4 Apple-native full spec
When its implementation status table is inspected
Then Phase F is marked landed for Makepad-rendered semantic controls while Phase G-I remain future work
Test: `rg "Phase F \\| Landed for Makepad-rendered semantic controls|Phase G-I \\| Future work" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`

### Scenario: no platform backend changes
Given this is a semantic surface step
When the git diff is inspected
Then no file under `platform/src` is changed
Test: `git diff --name-only HEAD -- platform/src`

### Scenario: affected crates compile
Given the semantic aliases are registered and used by aichat
When compilation is run
Then both widgets and aichat compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
