spec: task
name: "AI Chat Liquid Glass Step 91 Lower Scene Sibling Layer"
tags: [makepad, liquid-glass, apple-native, interleave, macos, layer-order]
---

## Intent

Move the visible `lower-scene-clear` probe from a sublayer of the primary
Makepad CAMetalLayer into its own sibling layer-backed NSView below the primary
Makepad view. This makes the probe a closer stand-in for the future LowerScene
surface in the Apple native interleave stack.

## Decisions

- Lifecycle drawable probing stays hidden on the primary CAMetalLayer sublayer.
- `LowerSceneClear` uses a sibling NSView positioned below the primary Makepad
  view, not a sublayer of the primary CAMetalLayer.
- The sibling probe still does not receive input and does not route Makepad draw
  passes.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not change normal `macos-native-clear` behavior.
- Do not change primary draw-pass routing.
- Do not expose `apple-native-interleave` as an enabled production backend.

## Acceptance Criteria

### Scenario: LowerSceneClear installs as a sibling view
Given the LowerScene probe must sit below native glass and below upper Makepad UI
When the macOS interleave installer is inspected
Then `LowerSceneClear` allocates an `NSView`, sets the probe CAMetalLayer as its layer, and adds it below the primary view
Test: `rg "install_lower_scene_probe_host_view|setLayer: probe_layer|positioned: -1i64|relativeTo: primary_view" platform/src/os/apple/macos/macos.rs`

### Scenario: lifecycle probe remains a hidden primary sublayer
Given existing lifecycle probing should stay behavior-compatible
When the macOS interleave installer is inspected
Then `LifecycleDrawable` still uses `addSublayer` and `setHidden: YES`
Test: `rg "install_lifecycle_probe_layer|addSublayer: probe_layer|setHidden: YES" platform/src/os/apple/macos/macos.rs`

### Scenario: probe resize updates both layer and host view
Given the sibling NSView owns the LowerScene CAMetalLayer
When Core Animation resizing code is inspected
Then it updates the optional probe host view frame as well as drawable size and contents scale
Test: `rg "host_view|setFrame: macos_surface_frame|setDrawableSize|setContentsScale" platform/src/os/apple/macos/macos.rs`

### Scenario: placement is visible in runtime logs
Given later validation needs to distinguish sublayer and sibling placements
When probe install logs are inspected
Then they include `placement=primary-sublayer` and `placement=sibling-below-primary`
Test: `rg "primary-sublayer|sibling-below-primary" platform/src/os/apple/macos/macos.rs`

### Scenario: missing superview downgrades to hidden
Given a sibling host cannot be inserted without a primary superview
When the LowerScene host installer is inspected
Then it reports `sibling-below-primary-no-superview` and returns hidden state
Test: `rg "sibling-below-primary-no-superview|, YES," platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given this changes AppKit view/layer setup
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only changes the macOS probe hierarchy
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-91-lower-scene-sibling-layer.spec platform/src/os/apple/macos/macos.rs "`
