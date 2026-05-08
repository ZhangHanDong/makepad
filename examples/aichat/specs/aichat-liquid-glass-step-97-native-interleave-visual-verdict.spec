spec: task
name: "AI Chat Liquid Glass Step 97 Native Interleave Visual Verdict"
tags: [makepad, aichat, liquid-glass, apple-native, interleave, manual-testing]
---

## Intent

Define the manual visual verdict gate for the Step 96 native interleave
prototype. Step 96 proves aichat can draw a lower scene pass into macOS final
composition; Step 97 tells a human tester what must be seen before the project
can treat AppleNativeInterleave as a production candidate.

## Decisions

- Store the verdict checklist in `examples/aichat/specs/`.
- Use the Studio runnable
  `makepad-example-aichat-macos-native-clear-interleave-lower-scene-pass`.
- A passing verdict requires recognizable native glass treatment over the lower
  scene, not merely a flat opaque background.
- A failing verdict keeps full interior glass on the ShaderBackdrop/hybrid route
  and keeps Apple native as underlay/substrate/control affordance work.

## Constraints

- Must not claim the visual verdict is complete.
- Must include the Step 96 log evidence required before visual judgment.
- Must include pass and fail criteria for native sampling.
- Must include a negative comparison against plain `macos-native-clear`.
- Must not modify source code.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-97-native-interleave-visual-verdict.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/src/**`
- `makepad.splash`

### Out of Scope

- Implementing production AppleNativeInterleave.
- Changing default backend selection.
- Running the human visual verdict to completion.

## Acceptance Criteria

Scenario: checklist names exact Studio targets
Test: `rg "makepad-example-aichat-macos-native-clear-interleave-lower-scene-pass|makepad-example-aichat-macos-native-clear|Makepad Studio|release" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
Given a tester starts the native interleave visual verdict
When they open the checklist
Then the exact release Studio targets are named

Scenario: checklist requires Step 96 logs
Test: `rg "LowerScenePass|native-lower-scene-pass=draw|lower-scene-role=draw|state=4|panels_installed=4" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
Given visual judgment is only meaningful after routing succeeds
When checklist coverage is searched
Then the required Step 96 log evidence is present

Scenario: checklist separates flat composition from glass sampling
Test: `rg "flat opaque background|blur|refraction|liquid|pass verdict|fail verdict|ShaderBackdrop" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
Given lower scene composition alone is not enough
When pass and fail criteria are inspected
Then the checklist distinguishes native treatment from a flat background

Scenario: checklist requires plain native negative comparison
Test: `rg "Compare against.*makepad-example-aichat-macos-native-clear|same underlay with an opaque scene inserted" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
Level: manual checklist coverage
Given lower-scene-pass must be judged against the existing underlay path
When comparison notes are inspected
Then the checklist requires a negative comparison against plain macos-native-clear

Scenario: checklist remains uncompleted
Test: `rg "\\[ \\]" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md && ! rg "\\[x\\]|\\[X\\]" examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md`
Given no human has completed the verdict in this step
When checkbox state is inspected
Then the checklist contains unchecked boxes and no checked boxes

Scenario: Step 97 remains documentation only
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-97-native-interleave-visual-verdict.spec examples/aichat/specs/aichat-liquid-glass-v4.2-native-interleave-visual-verdict.md "`
Given Step 97 only defines a visual verdict gate
When repository status is inspected
Then no source code or runnable files are changed
