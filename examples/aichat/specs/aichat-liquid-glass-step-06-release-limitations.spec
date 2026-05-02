spec: task
name: "AI Chat Liquid Glass Step 06 Release Limitations"
tags: [makepad, aichat, liquid-glass, release-notes, known-limitations]
---

## Intent

Document the current v4.1 Apple-native Liquid Glass implementation state for
reviewers and testers. The document must explain what runtime logs mean, what
State 4 does and does not prove, and which native Liquid Glass behaviors remain
explicitly out of scope for this phase.

## Decisions

- Add a dedicated v4.1 release/limitations document instead of rewriting the v1
  release notes.
- Record the compatibility log line and the new batch/container/panel log
  schema.
- State that Studio release runs are required for visual UI validation.
- Keep ShaderBackdrop documented as a separate future phase.
- List v4.1 non-goals exactly enough for testers to avoid reporting expected
  gaps as regressions.

## Constraints

- Must mention runtime switching, fullscreen, Stage Manager, multiple displays,
  and rounded-corner synchronization as known limitations.
- Must mention that native + shader backdrop mixing is rejected in one window in
  v4.1.
- Must not claim Studio visual validation has passed in this step.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-06-release-limitations.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/src/**`

### Out of Scope

- Code changes.
- Studio runtime validation.
- Screenshot capture.

## Acceptance Criteria

Scenario: v4.1 release notes document runtime state logs
Test: `rg "state=4|native-container|native-panel|panels_installed|panels_failed" examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
Given a reviewer needs to understand runtime state
When they read the v4.1 release notes
Then they can distinguish the v1 compatibility log from batch/container/panel logs

Scenario: known limitations are explicit
Test: `rg "runtime switching|fullscreen|Stage Manager|multiple displays|rounded-corner synchronization|native \\+ shader backdrop" examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
Given testers are validating v4.1
When they review known limitations
Then expected gaps are listed explicitly, including the rejected native + shader backdrop mix

Scenario: visual validation status is honest
Test: `rg "Studio release run required|not yet a passed visual validation|macOS 26" examples/aichat/specs/aichat-liquid-glass-v4.1-release-notes.md`
Given the local Studio bridge was unavailable during documentation
When the notes describe validation
Then they require a future Studio release run and do not claim visual pass

Scenario: Step 06 commit is documentation only
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-06-release-limitations.spec|examples/aichat/specs/aichat-liquid-glass-v4\\.1-release-notes.md)$" && exit 1 || exit 0`
Given only Step 06 files are staged
When the staged file list is checked
Then no source code files are included
