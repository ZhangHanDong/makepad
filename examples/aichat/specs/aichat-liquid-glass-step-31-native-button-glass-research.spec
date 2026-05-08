spec: task
name: "AI Chat Liquid Glass Step 31 Native Button Glass Research"
tags: [makepad, aichat, liquid-glass, apple-native, glass-controls, phase-f]
---

## Intent

Land the Phase F research deliverable required by
`aichat-liquid-glass-v4-apple-native-full.spec.md`: document how native
button/control glass should relate to `GlassButton`, `GlassToolbar`, and the
existing native panel descriptor path.

## Decisions

- Keep v4.1/v4.2 `GlassButton` and `GlassToolbar` Makepad-rendered above native
  panels.
- Do not model native buttons as `NativeGlassPanelDescriptor`.
- Treat native button glass as a later native-control backend, separate from
  panel/container descriptors.
- Record current local SDK evidence for AppKit `NSButton` and UIKit
  `UIButton.Configuration`.
- Record the iOS glass API blocker when the local SDK does not expose iOS 26
  glass types.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-31-native-button-glass-research.spec`
- `examples/aichat/specs/APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`

### Forbidden

- Do not change `platform/**`.
- Do not change `widgets/**`.
- Do not change `examples/aichat/src/**`.
- Do not claim iOS 26 typed API support unless it is validated against the local
  SDK.

## Acceptance Criteria

Scenario: research document exists and covers AppKit buttons
Test: `rg "NSButton|NSBezelStyle|bezelStyle|NSButtonCell" examples/aichat/specs/APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`
Given Phase F requires native button research
When the research document is inspected
Then it covers macOS AppKit button appearance options

Scenario: research document covers UIKit configuration
Test: `rg "UIButton.Configuration|UIButtonConfiguration|buttonWithConfiguration|configurationUpdateHandler|cornerStyle" examples/aichat/specs/APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`
Given Phase F requires iOS button research
When the research document is inspected
Then it covers UIKit configuration-based button options

Scenario: panel descriptors remain separate from native controls
Test: `rg "not.*NativeGlassPanelDescriptor|separate from panel descriptors|Makepad-rendered above native glass panels" examples/aichat/specs/APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`
Given native controls have different hit-testing and event routing
When the research document is inspected
Then it explicitly separates native button glass from panel descriptors

Scenario: local SDK evidence is recorded
Test: `rg "MacOSX15\\.5|iPhoneOS18\\.5|UIGlassEffect.*unavailable|UIGlassContainerEffect.*unavailable" examples/aichat/specs/APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`
Given the local SDK may not expose iOS 26 glass APIs
When the research document is inspected
Then it records the actual local SDK versions and blocker

Scenario: documentation-only change
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-31-native-button-glass-research.spec|examples/aichat/specs/APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md)$" && exit 1 || exit 0`
Given this is a research deliverable
When staged files are checked
Then no source files are included
