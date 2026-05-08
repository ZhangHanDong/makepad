spec: task
name: "AI Chat Liquid Glass Step 38 Glass Sidebar"
tags: [makepad, aichat, liquid-glass, glass-components, phase-f, sidebar]
---

## Intent

Add `GlassSidebar` as a semantic glass surface for navigation and persistent
workspace controls. This continues the Phase F shift from app-local `GlassPanel`
usage to reusable glass-aware components.

## Decisions

- Add `GlassSidebar` as a `GlassPanel` semantic alias.
- Use `GlassSidebar` for the aichat left navigation surface.
- Preserve the existing aichat native descriptor settings on the sidebar.
- Do not change nav button action handling in this step.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-38-glass-sidebar.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not change native panel descriptor collection.
- Do not add native AppKit/UIKit nav controls.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassSidebar widget alias is registered
Test: `rg "mod\\.widgets\\.GlassSidebar = mod\\.widgets\\.GlassPanel" widgets/src/glass_panel.rs`
Given aichat needs semantic glass surfaces
When the glass panel script module is registered
Then `GlassSidebar` is available as a Makepad-rendered glass panel alias

Scenario: aichat sidebar uses the semantic alias
Test: `rg "sidebar := GlassSidebar" examples/aichat/src/main.rs`
Given aichat renders a persistent left navigation surface
When the source is inspected
Then the sidebar is based on `GlassSidebar`

Scenario: v4 spec documents GlassSidebar behavior
Test: `rg "GlassSidebar|navigation surface|may opt into native descriptors" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given native descriptors remain opt-in
When the component section is inspected
Then `GlassSidebar` is described as a semantic surface, not native controls

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic component step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassSidebar` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
