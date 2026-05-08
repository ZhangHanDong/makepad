spec: task
name: "AI Chat Liquid Glass Step 40 Phase F Audit"
tags: [makepad, aichat, liquid-glass, glass-controls, phase-f, audit]
---

## Intent

Record the current Phase F semantic-control state before moving on to higher
risk work. The audit must map implemented widgets, aichat adoption, intentional
exceptions, and remaining follow-up tasks to concrete source evidence.

## Decisions

- Add a Phase F audit document under `examples/aichat/specs`.
- Treat app-local raw controls as acceptable only when they are explicitly
  classified.
- Keep this step documentation-only.
- Do not change UI/runtime/platform behavior.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-40-phase-f-audit.spec`
- `examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`

### Forbidden

- Do not change Rust source files.
- Do not change Makepad Studio run items.
- Do not change platform native glass backends.
- Do not change `AICHAT_GLASS_BACKEND` behavior.

## Acceptance Criteria

Scenario: Phase F audit document exists
Test: `test -f examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
Given Phase F semantic controls have several commits
When the specs directory is inspected
Then the audit document exists

Scenario: audit records implemented semantic widgets
Test: `rg "GlassSeparator|GlassToolbar|GlassButton|GlassIconButton|GlassPrimaryButton|GlassComposer|GlassSidebar|GlassCard" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
Given the audit should map source reality
When the audit is inspected
Then all implemented Phase F semantic widgets are named

Scenario: audit records remaining app-local exceptions
Test: `rg "ButtonFlat exceptions|GlassPanel exceptions|GlassSlider|nav_|copy_button|delete_button|cancel_button|app_shell|main_area" examples/aichat/specs/aichat-liquid-glass-phase-f-audit.md`
Given raw app-local widgets remain in aichat
When the audit is inspected
Then they are classified as exceptions or follow-up tasks

Scenario: documentation-only boundary is respected
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/aichat-liquid-glass-(step-40-phase-f-audit\\.spec|phase-f-audit\\.md)$" && exit 1 || exit 0`
Given this is an audit step
When the diff is inspected
Then only the audit spec and audit document are changed
