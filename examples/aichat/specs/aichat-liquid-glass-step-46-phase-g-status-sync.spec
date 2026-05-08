spec: task
name: "AI Chat Liquid Glass Step 46 Phase G Status Sync"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, docs]
---

## Intent

Synchronize the frozen v4 Apple-native spec with the current Phase G evidence:
iOS native glass is no longer completely untouched, but the real UIKit backend
is still blocked by local SDK availability.

## Decisions

- Mark Phase G as started, not landed.
- Reference the iOS explicit unsupported fallback and SDK evidence commits by
  artifact, not by mutable intent.
- Keep Phase H/I as future work.
- Keep the completion audit honest: Phase G cannot be complete until UIKit
  native panels are installed and visually validated.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not claim the UIKit backend is landed or supported.
- Do not change Phase A-F status.

## Acceptance Criteria

### Scenario: main spec marks Phase G as started
Given the v4 Apple-native full spec status table
When it is inspected
Then Phase G is marked started for unsupported fallback and SDK evidence
And Phase H-I remain future work
Test: `rg "Phase G \\| Started, backend blocked|aichat-liquid-glass-step-44-ios-native-glass-unsupported|aichat-liquid-glass-step-45-phase-g-sdk-evidence|Phase H-I \\| Future work" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`

### Scenario: main spec preserves UIKit backend acceptance
Given Phase G is not implemented yet
When the Phase G section is inspected
Then it still requires an iOS example with multiple native glass panels
Test: `rg "iOS example shows multiple native glass panels|rotation and keyboard do not leave stale frames|iPad split view and Stage Manager" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`

### Scenario: main spec does not overclaim UIKit support
Given the local SDK blocks the real UIKit backend
When the implementation status table is inspected
Then it does not mark Phase G as landed or supported
Test: `! rg "Phase G \\| Landed|Phase G \\| Supported|UIKit backend.*landed|UIKit backend.*supported" examples/aichat/specs/aichat-liquid-glass-v4-apple-native-full.spec.md`

### Scenario: documentation-only change
Given this is a status sync
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
