spec: task
name: "AI Chat Liquid Glass Step 08 Manual Visual Checklist"
tags: [makepad, aichat, liquid-glass, manual-testing, studio]
---

## Intent

Leave a concrete human visual validation checklist for the v4.1 Apple-native
Liquid Glass path. The checklist should tell a tester what to launch through
Makepad Studio, which logs to confirm, and which bright/dark wallpaper and UI
states still need direct visual judgment.

## Decisions

- Store the checklist in `examples/aichat/specs/`.
- Require Makepad Studio remote release runs for app launch validation.
- Cover both `macos-native` and `macos-native-clear` runnables.
- Include negative/fallback checks for shader mode and unsupported runtime.
- Keep the checklist as unchecked Markdown boxes so the human tester can mark
  results later.

## Constraints

- Must not claim the visual checklist has been completed.
- Must mention State 4 and v4.1 native container/panel logs.
- Must mention bright wallpaper, dark wallpaper, resize, input, Splash opaque
  guard, rounded corners, and screenshot capture.
- Must not modify source code.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-08-manual-checklist.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/src/**`

### Out of Scope

- Running Studio validation.
- Capturing screenshots.
- Fixing visual defects found by the checklist.

## Acceptance Criteria

Scenario: checklist names required Studio run targets
Test: `rg "makepad-example-aichat-macos-native|makepad-example-aichat-macos-native-clear|Makepad Studio|release" examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md`
Given a tester starts visual validation
When they open the checklist
Then they can identify the required Studio release run targets

Scenario: checklist covers logs and visual surfaces
Test: `rg "state=4|native-container|native-panel|bright wallpaper|dark wallpaper|resize|input|Splash|rounded corners|screenshot" examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md`
Given native glass requires human judgment
When checklist coverage is searched
Then logs and key visual/UI surfaces are present

Scenario: checklist remains uncompleted
Test: `rg "\\[ \\]" examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md && ! rg "\\[x\\]|\\[X\\]" examples/aichat/specs/aichat-liquid-glass-v4.1-manual-visual-checklist.md`
Given no human has completed validation in this step
When checkbox state is inspected
Then the checklist contains unchecked boxes and no checked boxes

Scenario: Step 08 commit is documentation only
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-08-manual-checklist.spec|examples/aichat/specs/aichat-liquid-glass-v4\\.1-manual-visual-checklist.md)$" && exit 1 || exit 0`
Given only Step 08 files are staged
When the staged file list is checked
Then no source code files are included
