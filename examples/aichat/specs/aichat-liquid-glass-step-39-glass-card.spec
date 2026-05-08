spec: task
name: "AI Chat Liquid Glass Step 39 Glass Card"
tags: [makepad, aichat, liquid-glass, glass-components, phase-f, card]
---

## Intent

Add `GlassCard` as the semantic readability surface for chat messages and
generated content. In the current aichat path it preserves the existing
Makepad-rendered rounded-card behavior instead of adding per-message native
panels.

## Decisions

- Add `GlassCard` as a `RoundedView` semantic alias.
- Use `GlassCard` for aichat `User` and `Assistant` message templates.
- Preserve the current local readability fill and radius.
- Do not create native panel descriptors per message row.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-39-glass-card.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not add per-row native views or native descriptors.
- Do not change Markdown, Splash, diagram, or message action behavior.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassCard widget alias is registered
Test: `rg "mod\\.widgets\\.GlassCard = mod\\.widgets\\.RoundedView" widgets/src/glass_panel.rs`
Given chat messages need a semantic readability surface
When the glass panel script module is registered
Then `GlassCard` is available as a Makepad-rendered rounded card alias

Scenario: aichat message templates use the semantic alias
Test: `rg "User := GlassCard|Assistant := GlassCard" examples/aichat/src/main.rs`
Given aichat renders user and assistant messages
When the source is inspected
Then both message templates are based on `GlassCard`

Scenario: v4 spec documents GlassCard as a readability surface
Test: `rg "GlassCard|readability surface|does not create per-message native panels" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given v4.1 must not create native views for every message row
When the component section is inspected
Then `GlassCard` is described as Makepad-rendered readability surface

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic component step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassCard` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
