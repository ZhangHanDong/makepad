spec: task
name: "AI Chat Liquid Glass Step 41 Glass Nav Button"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f, nav]
---

## Intent

Resolve the largest remaining Phase F app-local button group by adding
`GlassNavButton` for sidebar navigation rows and settings navigation.

## Decisions

- Add `GlassNavButton` as a `GlassButton` variant.
- Use it for aichat `nav_*` buttons and `settings_button`.
- Preserve active/inactive local styling through per-instance overrides.
- Keep navigation actions Makepad-rendered and non-native.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-41-glass-nav-button.spec`
- `examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change platform native glass backends.
- Do not change nav action handling.
- Do not add AppKit/UIKit native controls.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: GlassNavButton widget alias is registered
Test: `rg "mod\\.widgets\\.GlassNavButton = mod\\.widgets\\.GlassButton" widgets/src/glass_panel.rs`
Given sidebar navigation rows need a semantic glass-aware button
When the glass panel script module is registered
Then `GlassNavButton` is available as a Makepad-rendered button alias

Scenario: aichat sidebar navigation uses the semantic alias
Test: `rg "nav_(chat|appgen|search|plugins|automation|project) := GlassNavButton|settings_button := GlassNavButton" examples/aichat/src/main.rs`
Given aichat renders sidebar navigation
When the source is inspected
Then navigation rows and settings use `GlassNavButton`

Scenario: audit no longer lists nav buttons as raw ButtonFlat exceptions
Test: `rg "GlassNavButton.*implemented|Remaining ButtonFlat Exceptions" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md && ! rg "nav_chat|nav_appgen|nav_search|nav_plugins|nav_automation|nav_project|settings_button" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
Given the audit tracks raw button exceptions
When the audit is inspected
Then nav rows are no longer listed as remaining raw `ButtonFlat` exceptions

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic control step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassNavButton` is added and used by aichat
When affected crates are checked
Then widget registration and aichat script compilation succeed
