spec: task
name: "AI Chat Liquid Glass Step 50 Native Interleave Requirements"
tags: [makepad, liquid-glass, apple-native, interleave, renderer, docs]
---

## Intent

Document the renderer/platform requirements for a real `AppleNativeInterleave`
implementation based on the current Makepad Metal architecture. This keeps the
reserved backend name actionable without treating the existing underlay or
offscreen texture paths as sufficient evidence.

## Decisions

- A true native interleave requires two Makepad-rendered native surfaces around
  an Apple native glass view.
- Existing `DrawPassMode::Texture` is useful for offscreen rendering but is not
  by itself a foreground native surface.
- Studio screenshot and widget tooling need an explicit multi-surface story
  before production interleave can be accepted.
- Input remains Makepad-owned unless a later native-control phase defines
  forwarding.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not claim `AppleNativeInterleave` is implemented.
- Do not change the selected `ShaderBackdropInterior` route for aichat v4.2.

## Acceptance Criteria

### Scenario: requirements document names current renderer evidence
Given the AppleNativeInterleave requirements document
When it is inspected
Then it references `CAMetalLayer`, `DrawPassMode::MTKView`, and `DrawPassMode::Texture`
Test: `rg "CAMetalLayer|DrawPassMode::MTKView|DrawPassMode::Texture" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: requirements distinguish offscreen pass from native surface
Given offscreen texture passes already exist
When the requirements are inspected
Then they say offscreen texture rendering alone is not native interleave
Test: `rg "offscreen texture rendering alone is not native interleave|foreground native surface" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: requirements list implementation gates
Given `AppleNativeInterleave` is reserved
When the requirements are inspected
Then they list visual, input, Studio, resize/DPI, and lifecycle gates
Test: `rg "Visual gate|Input gate|Studio gate|Resize/DPI gate|Lifecycle gate" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`

### Scenario: completion audit references requirements
Given the completion audit tracks incomplete work
When it is inspected
Then it references the new renderer requirements artifact
Test: `rg "APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: documentation-only change
Given this is a requirements step
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
