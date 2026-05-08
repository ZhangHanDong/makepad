spec: task
name: "AI Chat Liquid Glass Step 42 Glass Utility Button"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f, utility]
---

## Intent

Close the remaining Phase F app-local raw button exceptions by adding
`GlassUtilityButton` for low-emphasis utility actions such as cancel, copy, and
delete.

## Decisions

- Add `GlassUtilityButton` as a `GlassButton` variant.
- Use it for aichat `cancel_button`, message `copy_button`, and message
  `delete_button`.
- Preserve local sizing and visibility behavior.
- Keep utility actions Makepad-rendered and non-native.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-42-glass-utility-button.spec`
- `examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not change button action handling.
- Do not add AppKit/UIKit native controls.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassUtilityButton widget alias is registered
Test: `rg "mod\\.widgets\\.GlassUtilityButton = mod\\.widgets\\.GlassButton" widgets/src/glass_panel.rs`
Given low-emphasis actions need a semantic glass-aware button
When the glass panel script module is registered
Then `GlassUtilityButton` is available as a Makepad-rendered button alias

Scenario: aichat utility actions use the semantic alias
Test: `rg "cancel_button := GlassUtilityButton|copy_button := GlassUtilityButton|delete_button := GlassUtilityButton" examples/aichat/src/main.rs`
Given aichat renders cancel/copy/delete utility actions
When the source is inspected
Then those buttons use `GlassUtilityButton`

Scenario: audit marks ButtonFlat exceptions closed
Test: `rg "Remaining ButtonFlat Exceptions|No remaining app-local ButtonFlat exceptions" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md && ! rg "cancel_button|copy_button|delete_button" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
Given the audit tracks raw button exceptions
When the audit is inspected
Then copy/delete/cancel are no longer listed as remaining raw `ButtonFlat` exceptions

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic control step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassUtilityButton` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
