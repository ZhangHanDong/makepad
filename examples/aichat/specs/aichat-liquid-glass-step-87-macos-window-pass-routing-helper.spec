spec: task
name: "AI Chat Liquid Glass Step 87 macOS Window Pass Routing Helper"
tags: [makepad, liquid-glass, apple-native, interleave, macos]
---

## Intent

Add the first private macOS draw-pass routing hook for future
`AppleNativeInterleave`: window draw passes should ask a helper which Metal
surface role they target. The helper still returns the current primary role, so
runtime behavior remains unchanged.

## Decisions

- Introduce `macos_surface_role_for_window_pass` as the single routing hook for
  window-backed draw passes.
- Keep all current window passes routed to `MacosMetalSurfaceRole::Primary`.
- Keep child and standalone draw passes on `DrawPassMode::Texture`.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not route any draw pass to `LowerScene` or `UpperUi` in this step.
- Do not render into the hidden interleave probe layer.
- Do not change Studio runnable behavior.

## Acceptance Criteria

### Scenario: window pass routing helper exists
Given macOS interleave needs a draw-pass routing boundary
When the macOS source is inspected
Then it defines `macos_surface_role_for_window_pass`
Test: `rg "fn macos_surface_role_for_window_pass" platform/src/os/apple/macos/macos.rs`

### Scenario: helper takes draw pass and window context
Given future routing needs both pass and window state
When the helper signature is inspected
Then it takes `DrawPassId` and `&MetalWindow`
Test: `rg "draw_pass_id: DrawPassId|metal_window: &MetalWindow" platform/src/os/apple/macos/macos.rs`

### Scenario: helper currently preserves primary routing
Given this step must not change runtime behavior
When the helper body is inspected
Then it returns `MacosMetalSurfaceRole::Primary`
Test: `rg "MacosMetalSurfaceRole::Primary" platform/src/os/apple/macos/macos.rs`

### Scenario: handle_repaint uses the helper
Given the helper is the routing boundary
When `handle_repaint` is inspected
Then it calls `macos_surface_role_for_window_pass` before drawing window passes
Test: `rg "macos_surface_role_for_window_pass\\(\\*draw_pass_id, metal_window\\)" platform/src/os/apple/macos/macos.rs`

### Scenario: non-window passes still render to textures
Given this step only changes the window-pass routing boundary
When draw-pass handling is inspected
Then `CxDrawPassParent::DrawPass` and `CxDrawPassParent::None` still use `DrawPassMode::Texture`
Test: `rg "CxDrawPassParent::DrawPass|CxDrawPassParent::None|DrawPassMode::Texture" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given the helper must not alter normal behavior
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only adds the macOS routing helper
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-87-macos-window-pass-routing-helper.spec platform/src/os/apple/macos/macos.rs "`
