spec: task
name: "AI Chat Liquid Glass Step 78 Completion Audit Refresh"
tags: [makepad, liquid-glass, apple-native, audit, phase-g]
---

## Intent

Refresh the completion audit after Steps 71-77 so it reflects the current iOS
installer skeleton, style overrides, selector coverage, shared descriptor diff
logic, iOS style events, and aichat iOS native event handling.

## Decisions

- Keep the objective marked incomplete.
- Replace stale wording that says the iOS backend still needs the explicit
  unsupported fallback to be replaced; that replacement has started with a
  runtime-gated installer skeleton.
- Make the next iOS gate runtime validation on iOS 26, not more local SDK-only
  scaffolding.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not change code in this step.
- Do not claim iOS runtime validation is done.
- Do not remove remaining Phase H / Phase I limitations.

## Acceptance Criteria

### Scenario: UIKit backend row mentions Steps 71-77
Given the completion audit tracks iOS backend state
When it is inspected
Then it mentions Steps 71, 72, 73, 74, 76, and 77
Test: `rg "Step 71|Step 72|Step 73|Step 74|Step 76|Step 77" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: next gates point to iOS 26 runtime validation
Given local scaffolding cannot prove UIKit visual behavior
When the Next Gates section is inspected
Then it calls for iOS 26 runtime validation of class availability, selector names, style raw values, visual output, rotation, safe area, keyboard, split view, and Stage Manager
Test: `rg "iOS 26 runtime validation|selector names|style raw values|visual output|rotation|safe area|keyboard|split view|Stage Manager" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: audit remains incomplete
Given the objective is still not fully achieved
When the audit is inspected
Then it still says current conclusion is not complete and keeps UIKit runtime validation incomplete
Test: `rg "Current conclusion: not complete|not runtime-validated beyond the dynamic installer skeleton" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: stale iOS unsupported-fallback gate is removed
Given the dynamic UIKit installer skeleton has started replacing the old unsupported fallback
When the Next Gates section is inspected
Then it no longer says the next step is to replace the explicit iOS unsupported fallback
Test: `! rg "replace the explicit iOS unsupported fallback" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: Phase H and Phase I limitations remain listed
Given advanced behavior and popup/modal work are still incomplete
When the audit is inspected
Then it still lists Phase H, Phase I, Stage Manager, multi-display, and popup/modal limitations
Test: `rg "Phase H is not implemented|Phase I is not implemented|Stage Manager and multi-display behavior remain known limitations|Popup/modal native glass remains a separate phase" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: no code changes
Given this is an audit refresh
When the worktree diff is inspected
Then no Rust source files are changed
Test: `! git diff --name-only | rg "\\.rs$"`

### Scenario: audit refresh only changes the allowed artifacts
Given this step is a documentation audit refresh
When the worktree diff is inspected
Then only the completion audit and this step spec are changed
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-completion-audit.md examples/aichat/specs/aichat-liquid-glass-step-78-completion-audit-refresh.spec "`
