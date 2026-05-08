spec: task
name: "AI Chat Liquid Glass Step 34 Glass Toolbar"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f, toolbar]
---

## Intent

Continue Phase F glass-aware controls by adding a semantic `GlassToolbar`.
In v4.1/v4.2 it must remain Makepad-rendered and must not create nested native
glass containers, because the native backend currently supports one container
per window.

## Decisions

- Add a `GlassToolbar` widget alias as a Makepad-rendered `GlassPanel` variant.
- Use `GlassToolbar` for the aichat top toolbar control groups.
- Keep toolbar grouping passive and non-native.
- Update the v4 Apple-native spec so `GlassToolbar` does not imply native
  container nesting in v4.1/v4.2.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-34-glass-toolbar.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not add a second native container.
- Do not make toolbar controls native interactive controls.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassToolbar widget alias is registered
Test: `rg "mod\\.widgets\\.GlassToolbar = mod\\.widgets\\.GlassPanel" widgets/src/glass_panel.rs`
Given Phase F needs a semantic toolbar grouping widget
When the glass panel script module is registered
Then `GlassToolbar` is available as a Makepad-rendered glass panel alias

Scenario: aichat top toolbar uses the semantic alias
Test: `rg "let ToolbarGlass = GlassToolbar" examples/aichat/src/main.rs`
Given aichat renders backend and opacity control groups
When the source is inspected
Then the local toolbar style is based on `GlassToolbar`

Scenario: v4 spec states toolbar native container nesting is deferred
Test: `rg "GlassToolbar remains Makepad-rendered|does not create nested native containers|multi-container phase" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given v4.1 allows one native container per window
When the GlassToolbar section is inspected
Then it does not promise native container nesting in the current phase

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic control step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassToolbar` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
