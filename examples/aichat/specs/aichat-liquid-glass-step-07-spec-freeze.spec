spec: task
name: "AI Chat Liquid Glass Step 07 Spec Freeze"
tags: [makepad, aichat, liquid-glass, spec-freeze, documentation]
---

## Intent

Close the loop between the frozen v4 Apple-native Liquid Glass design and the
Phase A-E work now landed on the branch. The spec set should clearly show which
phases are implemented, which evidence files and commits back them, and which
later phases remain future work.

## Decisions

- Add an implementation status section to the v4 full spec.
- Update Phase B/C/D plan documents with implemented evidence instead of
  changing their original scope.
- Reference the Step 01-06 task specs and the pushed commits as traceability
  evidence.
- Keep Phase F-I as future work.

## Constraints

- Must not change source code.
- Must not claim Studio visual validation has passed.
- Must keep ShaderBackdrop documented as a future, separate backend.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-07-spec-freeze.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- `examples/aichat/specs/aichat-liquid-glass-v4-phase-b.plan.md`
- `examples/aichat/specs/aichat-liquid-glass-v4-phase-c.plan.md`
- `examples/aichat/specs/aichat-liquid-glass-v4-phase-d.plan.md`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/src/**`

### Out of Scope

- New runtime behavior.
- Studio run or screenshot validation.
- Manual checklist creation.

## Acceptance Criteria

Scenario: v4 full spec records current implementation status
Test: `rg "Current Branch Implementation Status|Phase A|Phase B|Phase C|Phase D|Phase E|Studio visual validation remains pending" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given the v4 design document is the main reviewer entry point
When a reviewer reads it after this branch
Then it states Phase A-E status and that Studio visual validation remains pending

Scenario: Phase B/C/D plans include landed evidence
Test: `rg "Implementation Status|Landed evidence|Step 0[2-4]|cargo test|cargo check" examples/aichat/specs/aichat-liquid-glass-v4-phase-b.plan.md examples/aichat/specs/aichat-liquid-glass-v4-phase-c.plan.md examples/aichat/specs/aichat-liquid-glass-v4-phase-d.plan.md`
Given the phase plan documents remain in the repo
When they are searched for status
Then each plan points to landed evidence and verification commands

Scenario: future backend boundary remains explicit
Test: `rg "ShaderBackdrop.*future|Phase F|Phase G|Phase H|Phase I" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
Given v4.1 is AppleNative-only
When the frozen spec is searched
Then ShaderBackdrop and later phases remain future work

Scenario: Step 07 commit is documentation only
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-07-spec-freeze.spec|examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md|examples/aichat/specs/aichat-liquid-glass-v4-phase-b.plan.md|examples/aichat/specs/aichat-liquid-glass-v4-phase-c.plan.md|examples/aichat/specs/aichat-liquid-glass-v4-phase-d.plan.md)$" && exit 1 || exit 0`
Given only Step 07 files are staged
When the staged file list is checked
Then no source code files are included
