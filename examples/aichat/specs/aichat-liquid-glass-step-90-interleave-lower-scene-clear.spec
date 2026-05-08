spec: task
name: "AI Chat Liquid Glass Step 90 Interleave Lower Scene Clear"
tags: [makepad, liquid-glass, apple-native, interleave, macos, metal, probe]
---

## Intent

Add the first visible macOS interleave proof mode by clearing and presenting the
reserved LowerScene probe layer. This proves the extra CAMetalLayer can be
driven by Metal before real Makepad draw-pass routing is attached to it.

## Decisions

- `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-clear` is diagnostic only.
- The existing truthy lifecycle probe remains hidden and behavior-compatible.
- This step does not route any Makepad draw pass to LowerScene yet.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/metal.rs`
- `platform/src/os/apple/macos/macos.rs`
- `makepad.splash`
- this step spec

### Constraints

- Do not expose `apple-native-interleave` as an enabled production backend.
- Do not change primary window draw-pass routing.
- Do not draw Makepad UI into the LowerScene layer in this step.

## Acceptance Criteria

### Scenario: Metal can clear and present an arbitrary drawable
Given the LowerScene probe needs a Metal command buffer
When the Metal backend is inspected
Then `MetalCx` exposes a crate-private drawable clear helper
Test: `rg "pub\\(crate\\) fn clear_drawable|presentDrawable|renderCommandEncoderWithDescriptor" platform/src/os/apple/metal.rs`

### Scenario: lower-scene-clear drives the probe drawable
Given the probe mode is `LowerSceneClear`
When the macOS backend is inspected
Then it requests a probe drawable, clears it, presents it, and logs the result
Test: `rg "MacosInterleaveProbeMode::LowerSceneClear|clear_drawable|lower-scene-clear=presented|lower-scene-clear=failed" platform/src/os/apple/macos/macos.rs`

### Scenario: lifecycle probe remains hidden
Given the existing Studio runnable still injects `1`
When the probe layer installer is inspected
Then lifecycle mode still installs with `hidden=1`
Test: `rg "LifecycleDrawable.*YES|state=installed hidden=\\{\\}" platform/src/os/apple/macos/macos.rs`

### Scenario: Studio has a dedicated lower-scene-clear runnable
Given visual validation must use Studio RunItem
When Studio runnables are inspected
Then a dedicated runnable injects `lower-scene-clear`
Test: `rg "macos-native-clear-interleave-lower-scene-clear|lower-scene-clear" makepad.splash`

### Scenario: apple-native-interleave remains reserved
Given this proof is diagnostic only
When aichat backend resolution tests are inspected
Then `apple-native-interleave` still requires the renderer split
Test: `rg "aichat_apple_native_interleave_resolution_requires_renderer_split|AppleNativeInterleave.*requires_renderer_split" examples/aichat/src/main.rs`

### Scenario: nil drawable fails without presenting
Given `nextDrawable` can fail
When the Metal clear helper is inspected
Then it returns `false` for a nil drawable before command encoding
Test: `rg "if drawable == nil|return false" platform/src/os/apple/metal.rs`

### Scenario: platform crate still compiles
Given the change touches macOS Metal presentation
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only adds the lower-scene-clear proof path
When the worktree status is inspected
Then only the allowed files and this spec are changed, ignoring `.makepad/`
Level: repository state
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-90-interleave-lower-scene-clear.spec makepad.splash platform/src/os/apple/macos/macos.rs platform/src/os/apple/metal.rs "`
