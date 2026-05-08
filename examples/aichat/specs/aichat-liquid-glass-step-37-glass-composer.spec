spec: task
name: "AI Chat Liquid Glass Step 37 Glass Composer"
tags: [makepad, aichat, liquid-glass, glass-components, phase-f, composer]
---

## Intent

Move from low-level glass controls toward semantic aichat glass surfaces by
adding `GlassComposer`, the input surface that hosts text entry and composer
actions. The component stays Makepad-rendered and may opt into the existing
native underlay descriptor through normal `GlassPanel` properties.

## Decisions

- Add `GlassComposer` as a `GlassPanel` semantic alias.
- Use `GlassComposer` for the aichat composer surface.
- Preserve the existing aichat native descriptor settings on the composer.
- Do not change text input or button action routing.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-37-glass-composer.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not add AppKit/UIKit text input or button controls.
- Do not change native panel descriptor collection.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassComposer widget alias is registered
Test: `rg "mod\\.widgets\\.GlassComposer = mod\\.widgets\\.GlassPanel" widgets/src/glass_panel.rs`
Given aichat needs semantic glass surfaces
When the glass panel script module is registered
Then `GlassComposer` is available as a Makepad-rendered glass panel alias

Scenario: aichat composer uses the semantic alias
Test: `rg "composer := GlassComposer" examples/aichat/src/main.rs`
Given aichat renders the input composer surface
When the source is inspected
Then the composer is based on `GlassComposer`

Scenario: v4 spec documents GlassComposer behavior
Test: `rg "GlassComposer|input surface|may opt into native descriptors" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given native descriptors remain opt-in
When the component section is inspected
Then `GlassComposer` is described as a semantic surface, not a native text input

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic component step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassComposer` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
