spec: task
name: "AI Chat Liquid Glass Step 01 Phase A API Matrix"
tags: [makepad, aichat, liquid-glass, apple-native, phase-a, documentation]
---

## Intent

Document the Apple Liquid Glass API surface that Makepad will target before
runtime implementation continues. This task freezes the Phase A evidence files:
an API matrix, macOS and iOS probe snippets, and runtime notes that connect the
probe findings back to the v4 Apple-native Liquid Glass route.

## Decisions

- Keep the implementation-grade API matrix in
  `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`.
- Keep runtime notes in
  `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`.
- Store throwaway Apple API probes under
  `examples/aichat/prototypes/apple_liquid_glass/`.
- The probes are documentation artifacts only; they must not be compiled into
  Makepad crates or called from Rust runtime code.
- Phase A documents must mention macOS `NSGlassEffectView`,
  `NSGlassEffectContainerView`, iOS `UIGlassEffect`, and
  `UIGlassContainerEffect`.

## Constraints

- Must not change Rust runtime code in this task.
- Must keep this commit scoped to Phase A documentation and prototype files.
- Must keep the documents in Markdown so future phase docs can link to them.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-01-phase-a-api-matrix.spec`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- `examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md`
- `examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift`
- `examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift`

### Forbidden

- `platform/**`
- `widgets/**`
- `examples/aichat/src/**`
- `Cargo.toml`

### Out of Scope

- Runtime installation of native glass views.
- Makepad widget descriptor export.
- Studio visual validation.

## Acceptance Criteria

Scenario: Phase A API documents exist and cover Apple surface primitives
Test: `test -f examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md && test -f examples/aichat/specs/apple-liquid-glass-phase-a-runtime-notes.md && rg "NSGlassEffectView|NSGlassEffectContainerView|UIGlassEffect|UIGlassContainerEffect" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
Given the Phase A documentation files are present
When a reviewer searches the API matrix
Then the macOS and iOS Liquid Glass primitives are named explicitly

Scenario: Phase A probes stay isolated from Rust runtime
Test: `test -f examples/aichat/prototypes/apple_liquid_glass/macos/GlassContainerProbe.swift && test -f examples/aichat/prototypes/apple_liquid_glass/ios/UIKitGlassProbe.swift && git diff --name-only --cached | rg -v "^examples/aichat/(specs|prototypes)/" && exit 1 || exit 0`
Given the prototype Swift files are present
When the staged file list is checked
Then the Phase A commit does not include Rust runtime or widget files

Scenario: Phase A task does not modify implementation paths
Test: `git diff --name-only --cached | rg -v "^examples/aichat/(specs|prototypes)/" && exit 1 || exit 0`
Given only this task's files are staged
When the staged file list is checked
Then all staged paths are documentation or prototype paths under `examples/aichat`
