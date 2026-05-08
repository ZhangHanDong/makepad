spec: task
name: "AI Chat Liquid Glass Step 57 Scroll Edge Policy"
tags: [makepad, liquid-glass, apple-native, scroll-edge, phase-h]
---

## Intent

Define the scroll edge glass route before implementation. Phase H needs scroll
edge semantics, but v4.1 native glass still has one container per window,
passthrough native panels, and a strict native panel budget.

## Decisions

- Start scroll edge glass as Makepad semantic styling.
- Do not create additional native panels for scroll rows or individual messages.
- Keep scroll edge ownership with Makepad scroll widgets or wrappers until a
  platform-native toolbar/nav-bar behavior is explicitly designed.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not claim scroll edge glass is implemented.
- Do not increase the native panel budget.

## Acceptance Criteria

### Scenario: scroll edge route is explicit
Given scroll edge glass remains Phase H work
When the scroll edge policy is inspected
Then it says the next implementation starts as Makepad semantic style, not additional native panels
Test: `rg "Makepad semantic style|not as additional native panels|must not create native panels" examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`

### Scenario: native panel budget remains protected
Given native panels are capped in v4.1
When the scroll edge policy is inspected
Then it references the native panel budget and rejects per-row or per-message native panels
Test: `rg "12 visible panels|Per-row|row, message|native panel budget" examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`

### Scenario: future native gate is documented
Given native scroll edge behavior is not implemented
When the policy is inspected
Then it lists future ownership and platform-native questions
Test: `rg "Future Native Gate|Which widget owns scroll edge state|platform-native scroll edge" examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`

### Scenario: completion audit references scroll edge policy
Given full Apple native support remains incomplete
When the completion audit is inspected
Then it references `APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`
Test: `rg "APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: documentation-only change
Given this is a policy step
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
