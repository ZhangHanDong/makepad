spec: task
name: "AI Chat Liquid Glass Step 36 Glass Button Variants"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f, button]
---

## Intent

Extend the Phase F button semantic layer with icon and prominent button
variants so composer controls no longer depend only on app-local `ButtonFlat`
styles.

## Decisions

- Add `GlassIconButton` for compact square composer actions.
- Add `GlassPrimaryButton` for prominent commit/send actions.
- Use both variants in aichat's local composer styles.
- Keep both variants Makepad-rendered and non-native in v4.1/v4.2.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-36-glass-button-variants.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not add AppKit/UIKit button controls.
- Do not change button action handling.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: glass button variants are registered
Test: `rg "mod\\.widgets\\.GlassIconButton = mod\\.widgets\\.GlassButton|mod\\.widgets\\.GlassPrimaryButton = mod\\.widgets\\.GlassButton" widgets/src/glass_panel.rs`
Given composer controls need icon and prominent button semantics
When the glass panel script module is registered
Then both variants are available as Makepad-rendered aliases

Scenario: aichat composer styles use semantic variants
Test: `rg "let IconButton = GlassIconButton|let SendButton = GlassPrimaryButton" examples/aichat/src/main.rs`
Given aichat renders composer action buttons
When the source is inspected
Then icon and send button local styles are based on glass-aware variants

Scenario: v4 spec documents the variants as non-native
Test: `rg "GlassIconButton|GlassPrimaryButton|Makepad-rendered variants" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given native interactive controls are deferred
When the GlassButton section is inspected
Then icon and prominent variants are described as Makepad-rendered controls

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic control step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given button variants are added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
