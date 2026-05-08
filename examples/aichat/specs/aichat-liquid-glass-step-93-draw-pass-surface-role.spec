spec: task
name: "AI Chat Liquid Glass Step 93 Draw Pass Surface Role"
tags: [makepad, liquid-glass, apple-native, interleave, draw-pass, api]
---

## Intent

Introduce a platform-neutral draw-pass surface role so future aichat work can
mark background/scene passes as `LowerScene` and keep text/input passes on the
primary/upper surface. This is the API scaffold required to replace the
diagnostic whole-window mirror route with real interleave splitting.

## Decisions

- Existing passes default to `DrawPassSurfaceRole::Primary`.
- The role is exposed through Rust `DrawPass` methods and `ScriptDrawPass`.
- macOS maps `DrawPassSurfaceRole` into its private `MacosMetalSurfaceRole`.

## Boundaries

### Allowed Changes

- `platform/src/draw_pass.rs`
- `platform/src/lib.rs`
- `platform/src/script/draw.rs`
- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not change existing pass behavior for default `Primary` passes.
- Do not wire aichat UI to `LowerScene` in this step.
- Do not expose AppKit or Metal types in the public API.

## Acceptance Criteria

### Scenario: draw pass role enum exists
Given native interleave needs a platform-neutral routing marker
When draw pass API is inspected
Then `DrawPassSurfaceRole` has `Primary`, `LowerScene`, and `UpperUi`
Test: `rg "enum DrawPassSurfaceRole|Primary|LowerScene|UpperUi" platform/src/draw_pass.rs`

### Scenario: role defaults to Primary
Given existing apps must keep their current rendering behavior
When `CxDrawPass::default` is inspected
Then `surface_role` defaults to `DrawPassSurfaceRole::Primary`
Test: `rg "surface_role: DrawPassSurfaceRole::Primary|surface_role: DrawPassSurfaceRole" platform/src/draw_pass.rs`

### Scenario: Rust API can set and read the role
Given future app code needs to assign a pass role
When `DrawPass` methods are inspected
Then it exposes `set_surface_role` and `surface_role`
Test: `rg "set_surface_role|surface_role\\(&self" platform/src/draw_pass.rs`

### Scenario: script API carries the role
Given aichat UI uses script-defined passes
When script draw-pass bindings are inspected
Then `ScriptDrawPass` has a live `surface_role` and the draw module exports `DrawPassSurfaceRole`
Test: `rg "surface_role|DrawPassSurfaceRole" platform/src/draw_pass.rs platform/src/script/draw.rs`

### Scenario: macOS routing reads the role
Given macOS interleave routing needs platform-private roles
When macOS repaint routing is inspected
Then `macos_surface_role_for_window_pass` maps `DrawPassSurfaceRole` into `MacosMetalSurfaceRole`
Test: `rg "DrawPassSurfaceRole|macos_surface_role_for_window_pass|MacosMetalSurfaceRole::LowerScene|MacosMetalSurfaceRole::UpperUi" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given this changes public draw-pass API
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only adds draw-pass role API and macOS mapping
When the worktree status is inspected
Then only the allowed files and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-93-draw-pass-surface-role.spec platform/src/draw_pass.rs platform/src/lib.rs platform/src/os/apple/macos/macos.rs platform/src/script/draw.rs "`
