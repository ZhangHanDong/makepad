spec: task
name: "AI Chat Liquid Glass Step 96 aichat LowerScene Pass Probe"
tags: [makepad, aichat, liquid-glass, apple-native, interleave, lower-scene]
---

## Intent

Add the first aichat-owned LowerScene draw pass probe. Unlike
`lower-scene-role`, this mode does not force the main window pass into
LowerScene. Instead, aichat records a dedicated procedural scene pass with
`DrawPassSurfaceRole::LowerScene` while the normal UI pass remains primary.

## Decisions

- `lower-scene-pass` installs the sibling LowerScene surface without forcing all
  window passes to LowerScene.
- aichat reuses its procedural backdrop scene shader for the probe pass.
- This is still diagnostic; production UI is not yet split into final scene/UI
  passes.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- `examples/aichat/src/main.rs`
- `makepad.splash`
- this step spec

### Constraints

- Do not change normal `macos-native-clear` behavior.
- Do not change shader backdrop proof behavior.
- Do not make `apple-native-interleave` a supported backend.

## Acceptance Criteria

### Scenario: lower-scene-pass probe mode exists
Given aichat needs a role-based pass probe without forcing the main pass
When macOS probe mode parsing is inspected
Then `lower-scene-pass` maps to `MacosInterleaveProbeMode::LowerScenePass`
Test: `rg "lower-scene-pass|LowerScenePass" platform/src/os/apple/macos/macos.rs`

### Scenario: aichat records a LowerScene pass
Given the lower-scene-pass probe is enabled
When aichat draw code is inspected
Then it sets `DrawPassSurfaceRole::LowerScene`, parents the pass to `main_window`, and draws the scene shader
Test: `rg "native_lower_scene_pass_probe_enabled|DrawPassSurfaceRole::LowerScene|CxDrawPassParent::Window|native-lower-scene-pass=draw" examples/aichat/src/main.rs`

### Scenario: Studio has a dedicated lower-scene-pass runnable
Given runtime validation must use Studio RunItem
When Studio runnables are inspected
Then a dedicated runnable injects `lower-scene-pass`
Test: `rg "macos-native-clear-interleave-lower-scene-pass|lower-scene-pass" makepad.splash`

### Scenario: aichat still compiles
Given this records an app-owned pass
When aichat is checked
Then script/app compilation succeeds
Test: `cargo check -p makepad-example-aichat --release`

### Scenario: no unrelated files are changed
Given this step only adds the aichat LowerScene pass probe
When the worktree status is inspected
Then only the allowed files and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-96-aichat-lower-scene-pass-probe.spec examples/aichat/src/main.rs makepad.splash platform/src/os/apple/macos/macos.rs "`
