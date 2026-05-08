spec: task
name: "AI Chat Liquid Glass Step 94 macOS LowerScene Role Route"
tags: [makepad, liquid-glass, apple-native, interleave, macos, draw-routing]
---

## Intent

Route macOS window draw passes marked `DrawPassSurfaceRole::LowerScene` into
the LowerScene interleave surface when one is installed. This is the first
non-mirror role-based routing path toward a real Apple native glass split.

## Decisions

- `LowerScene` passes draw to the LowerScene sibling surface and skip primary
  drawing when the LowerScene drawable is available.
- If no LowerScene drawable is available, routing falls back to primary drawing
  so role-marked passes do not disappear.
- `Primary` and `UpperUi` passes keep the existing primary-layer path.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not wire aichat UI to `LowerScene` in this step.
- Do not change default `Primary` pass behavior.
- Do not remove the diagnostic `lower-scene-mirror` route yet.

## Acceptance Criteria

### Scenario: LowerScene role has a dedicated drawable path
Given a window pass is marked `DrawPassSurfaceRole::LowerScene`
When macOS repaint routing is inspected
Then it requests `next_lower_scene_role_drawable` and draws to `DrawPassMode::Drawable(lower_scene_drawable)`
Test: `rg "next_lower_scene_role_drawable|DrawPassMode::Drawable\\(lower_scene_drawable\\)|lower-scene-role=draw" platform/src/os/apple/macos/macos.rs`

### Scenario: LowerScene success skips primary drawing
Given a LowerScene drawable is available
When macOS repaint routing is inspected
Then the LowerScene branch continues after drawing instead of falling through to the primary drawable path
Test: `rg "MacosMetalSurfaceRole::LowerScene =>|continue;" platform/src/os/apple/macos/macos.rs`

### Scenario: LowerScene failure falls back to primary
Given a LowerScene drawable can be unavailable
When macOS repaint routing is inspected
Then failure logs `lower-scene-role=failed` and primary drawing remains reachable
Test: `rg "lower-scene-role=failed|draw_primary_window_pass" platform/src/os/apple/macos/macos.rs`

### Scenario: Primary and UpperUi remain primary layer routes
Given existing window passes default to Primary and future UI passes use UpperUi
When macOS repaint routing is inspected
Then `Primary` and `UpperUi` both call the primary draw helper
Test: `rg "MacosMetalSurfaceRole::Primary|MacosMetalSurfaceRole::UpperUi|draw_primary_window_pass" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given this changes macOS repaint routing
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: aichat still compiles
Given aichat uses the default primary window pass
When aichat is checked
Then the new routing does not break script/app compilation
Test: `cargo check -p makepad-example-aichat --release`

### Scenario: no unrelated files are changed
Given this step only changes macOS role routing
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-94-macos-lower-scene-role-route.spec platform/src/os/apple/macos/macos.rs "`
