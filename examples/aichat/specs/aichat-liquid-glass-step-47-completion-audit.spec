spec: task
name: "AI Chat Liquid Glass Step 47 Completion Audit"
tags: [makepad, liquid-glass, apple-native, audit, roadmap]
---

## Intent

Create a durable completion audit for the objective "complete Apple native
Liquid Glass API support" so future work can distinguish landed evidence,
blocked platform work, and deferred advanced behavior.

## Decisions

- Treat complete support as a checklist, not as a single State 4 log.
- Map each checklist item to concrete files, commands, or runtime evidence.
- Mark uncertainty as incomplete.
- Keep the current goal open because UIKit backend, native interleave, and
  advanced native behavior are not done.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not mark the objective complete.
- Do not claim full native interior Liquid Glass is implemented.

## Acceptance Criteria

### Scenario: audit restates objective as deliverables
Given the completion audit document
When it is inspected
Then it lists concrete deliverables for macOS, UIKit, controls, advanced native behavior, and validation
Test: `rg "Objective Restatement|macOS AppKit backend|UIKit backend|Native interactive controls|Advanced native behavior|Validation gates" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: audit maps artifacts to evidence
Given previous phase artifacts exist
When the completion audit is inspected
Then it references Phase A-F evidence, Step 44, Step 45, and the Studio State 4 run evidence
Test: `rg "APPLE-LIQUID-GLASS-API-MATRIX|aichat-liquid-glass-phase-f-audit|aichat-liquid-glass-step-44-ios-native-glass-unsupported|aichat-liquid-glass-step-45-phase-g-sdk-evidence|state=4|panels_installed=4" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: audit lists incomplete requirements
Given the goal is not complete
When the completion audit is inspected
Then it explicitly lists UIKit backend, native interleave/full interior glass, Phase H, and Phase I as incomplete
Test: `rg "Incomplete Requirements|UIKit backend is not implemented|full native interior Liquid Glass is not implemented|Phase H is not implemented|Phase I is not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: audit does not overclaim completion
Given the objective remains open
When the completion audit is inspected
Then it states the goal is not complete
And it does not contain "goal complete"
Test: `rg "Current conclusion: not complete" examples/aichat/specs/aichat-liquid-glass-completion-audit.md && ! rg "goal complete|objective complete" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: documentation-only change
Given this is an audit step
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
