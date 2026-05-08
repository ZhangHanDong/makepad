spec: task
name: "AI Chat Liquid Glass Step 81 macOS Surface Role Scaffold"
tags: [makepad, liquid-glass, apple-native, interleave, macos]
---

## Intent

Add the first no-behavior-change renderer scaffold for future
`AppleNativeInterleave`: macOS `MetalWindow` instances explicitly record that
their current Metal surface role is the single primary surface.

## Decisions

- The only active role in this step is `MacosMetalSurfaceRole::Primary`.
- Future lower/upper split roles are documented in the enum comment but not
  constructed or exposed yet.
- Window and popup creation must both initialize the role to `Primary`.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not change rendering, draw-pass routing, native glass installation, or
  popup behavior.
- Do not expose `AppleNativeInterleave` as an available backend.
- Keep this scaffold private to the macOS platform module.

## Acceptance Criteria

### Scenario: macOS surface role type exists
Given the macOS renderer still has one active surface
When the platform source is inspected
Then it defines `MacosMetalSurfaceRole` with a `Primary` role
Test: `rg "enum MacosMetalSurfaceRole|Primary" platform/src/os/apple/macos/macos.rs`

### Scenario: future lower and upper roles are documented but inactive
Given lower and upper surfaces are not implemented in this step
When the enum comment is inspected
Then it documents future lower scene and upper UI roles without adding active variants
Test: `rg "lower scene and upper UI roles once draw routing exists" platform/src/os/apple/macos/macos.rs`

### Scenario: MetalWindow stores the surface role
Given each macOS window owns the current primary layer
When the `MetalWindow` struct is inspected
Then it has a `surface_role: MacosMetalSurfaceRole` field
Test: `rg "surface_role: MacosMetalSurfaceRole" platform/src/os/apple/macos/macos.rs`

### Scenario: normal windows initialize the primary role
Given normal windows are still single-surface windows
When `MetalWindow::new` is inspected
Then it initializes `surface_role` to `MacosMetalSurfaceRole::Primary`
Test: `rg "surface_role: MacosMetalSurfaceRole::Primary" platform/src/os/apple/macos/macos.rs`

### Scenario: popup windows initialize the primary role
Given popup windows still use the existing single-surface path
When `MetalWindow::new_popup` is inspected
Then it also initializes `surface_role` to `MacosMetalSurfaceRole::Primary`
Test: `test "$(rg -c "surface_role: MacosMetalSurfaceRole::Primary" platform/src/os/apple/macos/macos.rs)" = "2"`

### Scenario: platform crate still compiles
Given the scaffold must not change runtime behavior
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only adds the private macOS role scaffold
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-81-macos-surface-role.spec platform/src/os/apple/macos/macos.rs "`
