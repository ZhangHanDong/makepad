spec: task
name: "AI Chat Liquid Glass Step 32 Glass Separator"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f]
---

## Intent

Start Phase F glass-aware controls with the lowest-risk semantic component:
`GlassSeparator`. Separators should no longer be only app-local hardcoded
`SolidView` lines; they should have a reusable glass-aware widget identity that
can later adapt to reduce transparency and increased contrast.

## Decisions

- Add Makepad-rendered `GlassSeparator` and `GlassVSeparator` widget aliases.
- Keep them passive and non-native; they are not `NativeGlassPanelDescriptor`s.
- Use `GlassVSeparator` for the aichat sidebar/main divider.
- Do not change native platform backends.
- Do not add native button/control routing in this step.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-32-glass-separator.spec`
- `widgets/src/glass_panel.rs`
- `examples/aichat/src/main.rs`

### Forbidden

- Do not change `platform/**`.
- Do not add AppKit/UIKit control APIs.
- Do not change the default native panel behavior.

## Acceptance Criteria

Scenario: GlassSeparator widgets are registered
Test: `rg "mod\\.widgets\\.GlassSeparator|mod\\.widgets\\.GlassVSeparator" widgets/src/glass_panel.rs`
Given widgets are loaded
When the glass panel script module is registered
Then horizontal and vertical glass separator aliases are available

Scenario: aichat uses semantic vertical separator
Test: `rg "GlassVSeparator|draw_bg \\+: \\{\\s*color: #xEAD8B81E" examples/aichat/src/main.rs`
Given aichat renders the sidebar/main divider
When the source is inspected
Then it uses `GlassVSeparator` instead of a raw `SolidView` line

Scenario: no platform backend changes
Test: `git diff --name-only HEAD -- platform/src`
Given this is a Phase F semantic control step
When the diff is inspected
Then no platform source files are listed

Scenario: affected crates compile
Test: `cargo check -p makepad-widgets && cargo check -p makepad-example-aichat`
Given `GlassSeparator` aliases are added
When affected crates are checked
Then widget registration and aichat script compilation succeed
