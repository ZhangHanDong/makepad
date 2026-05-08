spec: task
name: "AI Chat Liquid Glass Step 35 Glass Button"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f, button]
---

## Intent

Continue Phase F glass-aware controls with a reusable `GlassButton` semantic
alias. In v4.1/v4.2 it remains a Makepad-rendered button above native underlay
panels and does not use native interactive AppKit/UIKit controls.

## Decisions

- Add `GlassButton` as a `ButtonFlat` variant with glass-aware colors.
- Use `GlassButton` as the base for aichat pill buttons.
- Keep icon buttons and the prominent send button as local follow-up styles.
- Do not add native control hit-testing or forwarding in this step.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-35-glass-button.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not add native interactive controls.
- Do not change button event routing.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassButton widget alias is registered
Test: `rg "mod\\.widgets\\.GlassButton = mod\\.widgets\\.ButtonFlat" widgets/src/glass_panel.rs`
Given Phase F needs semantic glass-aware controls
When the glass panel script module is registered
Then `GlassButton` is available as a Makepad-rendered button alias

Scenario: aichat pill buttons use the semantic alias
Test: `rg "let PillButton = GlassButton" examples/aichat/src/main.rs`
Given aichat renders pill-shaped composer controls
When the source is inspected
Then the local pill style is based on `GlassButton`

Scenario: v4 spec states GlassButton remains Makepad-rendered
Test: `rg "GlassButton remains Makepad-rendered|first implementation is a semantic ButtonFlat variant|native interactive controls are deferred" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given native controls need a separate hit-test policy
When the GlassButton section is inspected
Then it does not imply AppKit/UIKit button routing in v4.1/v4.2

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic control step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassButton` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
