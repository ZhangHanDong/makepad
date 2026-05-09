spec: task
name: "AI Chat Liquid Glass Step 98 Native Interleave Fail Verdict"
tags: [makepad, aichat, liquid-glass, apple-native, interleave, manual-testing]
---

## Intent

Record the manual visual verdict for the Step 96/97 native interleave
prototype. The user observed a gray/grid lower scene without transparency,
refraction, or liquid distortion, so this route must not be promoted to a
production full-native Liquid Glass implementation.

## Decisions

- Treat the May 9, 2026 manual verdict as a fail verdict for production
  `AppleNativeInterleave`.
- Keep `AppleNativeInterleave` as a prototype/diagnostic route only.
- Keep complete aichat interior glass work on the ShaderBackdrop or hybrid
  route unless a later native sampling probe produces a different visual
  verdict.
- Do not remove the Step 96 runnable or lower scene pass probe; it remains
  useful evidence and a regression target.

## Constraints

- Must not modify source code.
- Must not delete the native interleave probe files.
- Must not claim full native Liquid Glass is complete.
- Must update the visual verdict checklist and completion audit.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-98-native-interleave-fail-verdict.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/src/**`
- `makepad.splash`

## Acceptance Criteria

Scenario: fail verdict is recorded
Test: `rg "Manual Verdict.*2026-05-09|Fail verdict|gray/grid|no transparency|no recognizable blur/refraction/liquid distortion" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
Given the user reported the candidate visual result
When the verdict checklist is inspected
Then the fail verdict and observed symptoms are recorded

Scenario: route decision does not promote native interleave
Test: `rg "AppleNativeInterleave.*prototype|not accepted for production|ShaderBackdrop|hybrid" examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
Given the native interleave visual verdict failed
When route documentation is inspected
Then AppleNativeInterleave remains a prototype and ShaderBackdrop/hybrid remains the complete interior route

Scenario: source code remains untouched
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-completion-audit.md examples/aichat/specs/aichat-liquid-glass-step-98-native-interleave-fail-verdict.spec examples/aichat/specs/aichat-liquid-glass-v4.2-compositing-redesign.spec.md examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md "`
Given this step only records the manual verdict
When repository status is inspected
Then no source code or runnable files are changed
