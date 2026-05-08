spec: task
name: "AI Chat Liquid Glass Step 92 Lower Scene Mirror Route"
tags: [makepad, liquid-glass, apple-native, interleave, macos, draw-routing]
---

## Intent

Add the first diagnostic Makepad draw-pass route into the macOS LowerScene
interleave surface. The route mirrors the current window pass into the
LowerScene probe drawable before drawing the normal primary layer, proving that
Makepad rendering can target the lower native surface.

## Decisions

- `lower-scene-mirror` is diagnostic only and must not expose
  `apple-native-interleave` as production-supported.
- The route mirrors the existing window pass; it does not yet split scene
  content from upper UI.
- Normal primary drawing still runs after the LowerScene mirror draw.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- `makepad.splash`
- this step spec

### Constraints

- Do not change normal `macos-native-clear` behavior.
- Do not change shader backdrop behavior.
- Do not add widget API for pass roles in this step.

## Acceptance Criteria

### Scenario: lower-scene-mirror env mode exists
Given a diagnostic draw-route proof needs an explicit mode
When macOS env parsing is inspected
Then `lower-scene-mirror` maps to `MacosInterleaveProbeMode::LowerSceneMirror`
Test: `rg "lower-scene-mirror|LowerSceneMirror" platform/src/os/apple/macos/macos.rs`

### Scenario: mirror mode routes a drawable before primary drawing
Given the probe is in `LowerSceneMirror`
When repaint routing is inspected
Then it requests a LowerScene drawable and calls `draw_pass` with that drawable before the primary drawable path
Level: static routing guard plus Studio runtime log
Test: `rg "next_lower_scene_mirror_drawable|DrawPassMode::Drawable\\(lower_scene_drawable\\)|lower-scene-mirror=draw" platform/src/os/apple/macos/macos.rs`

### Scenario: primary drawing remains intact
Given mirror routing is diagnostic
When repaint routing is inspected
Then the normal primary `nextDrawable` and primary `draw_pass` path remains after the mirror route
Level: static routing guard
Test: `rg "metal_window.ca_layer, nextDrawable|DrawPassMode::Drawable\\(drawable\\)|DrawPassMode::Resizing\\(drawable\\)" platform/src/os/apple/macos/macos.rs`

### Scenario: mirror drawable failure is logged once
Given `nextDrawable` can fail on the LowerScene probe
When mirror drawable routing is inspected
Then the failure path logs `lower-scene-mirror=failed`
Test: `rg "lower-scene-mirror=failed|lower_scene_route_logged" platform/src/os/apple/macos/macos.rs`

### Scenario: Studio has a dedicated mirror runnable
Given runtime validation must use Studio RunItem
When Studio runnables are inspected
Then a dedicated runnable injects `lower-scene-mirror`
Test: `rg "macos-native-clear-interleave-lower-scene-mirror|lower-scene-mirror" makepad.splash`

### Scenario: platform crate still compiles
Given this changes draw-pass routing
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only adds the diagnostic lower-scene mirror route
When the worktree status is inspected
Then only the allowed files and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-92-lower-scene-mirror-route.spec makepad.splash platform/src/os/apple/macos/macos.rs "`
