spec: task
name: "AI Chat Liquid Glass Step 52 Advanced Behavior Gates"
tags: [makepad, liquid-glass, apple-native, phase-h, phase-i, docs]
---

## Intent

Turn the remaining Phase H and Phase I native Liquid Glass limitations into
concrete implementation gates. The goal is to prevent fullscreen, multi-display,
Stage Manager, inactive-window behavior, animated morphing, and popup/modal
glass from being treated as complete merely because the main-window
`AppleNativeUnderlay` path reaches State 4.

## Decisions

- Keep Phase H and Phase I marked incomplete.
- Treat `state=4` main-window underlay evidence as insufficient for advanced
  native behavior.
- Require explicit fullscreen, multi-display, inactive-window, and transient
  window validation before claiming complete Apple-native Liquid Glass support.
- Keep popup/modal native glass separate from main-window native panel work.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not claim Phase H or Phase I is implemented.
- Do not change the current `AppleNativeUnderlay` runtime path.

## Acceptance Criteria

### Scenario: advanced behavior gates are explicit
Given Phase H and Phase I remain incomplete
When the advanced behavior gates document is inspected
Then it lists fullscreen, multi-display, Stage Manager, inactive-window behavior, animated spacing, scroll edge glass, and popup/modal native glass
Test: `rg "Fullscreen|Multi-display|Stage Manager|Inactive window|Animated spacing|Scroll edge glass|Popup/modal" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`

### Scenario: non-completion signals prevent false positives
Given the current macOS underlay can reach State 4
When the advanced behavior gates document is inspected
Then it says State 4 for the main window underlay is not enough to claim Phase H or Phase I support
Test: `rg "state=4|not enough to claim Phase H or Phase I" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`

### Scenario: completion audit references advanced behavior gates
Given the full objective remains incomplete
When the completion audit is inspected
Then it references `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
Test: `rg "APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: documentation-only change
Given this is a gate-definition step
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
