spec: task
name: "AI Chat Liquid Glass Step 83 macOS Interleave Layer Probe"
tags: [makepad, liquid-glass, apple-native, interleave, macos, probe]
---

## Intent

Add a hidden, opt-in macOS layer probe for future `AppleNativeInterleave`
work. The probe creates a second `CAMetalLayer` using the shared layer factory
and attaches it inside the existing primary layer hierarchy without drawing to
it or exposing the backend as available.

## Decisions

- The probe is disabled by default and only enabled by
  `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=1|true|on`.
- The probe layer is hidden and not used by draw-pass routing.
- Resize/DPI updates must keep the probe layer frame, drawable size, and
  content scale aligned with the primary layer.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not render into the probe layer.
- Do not expose `AppleNativeInterleave` as an available backend.
- Do not alter default window, popup, shader, native-underlay, or screenshot
  behavior when the env var is absent.

## Acceptance Criteria

### Scenario: probe is controlled by an explicit environment flag
Given the probe must be off by default
When the macOS source is inspected
Then it reads `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE` and accepts `1`, `true`, or `on`
Test: `rg "AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE|Some\\(\"1\"\\)|Some\\(\"true\"\\)|Some\\(\"on\"\\)" platform/src/os/apple/macos/macos.rs`

### Scenario: probe layer is created from the shared layer factory
Given interleave layers must share primary layer configuration
When the macOS source is inspected
Then the probe calls `new_macos_ca_metal_layer`
Test: `rg "install_native_interleave_layer_probe|new_macos_ca_metal_layer\\(metal_cx, delegate\\)" platform/src/os/apple/macos/macos.rs`

### Scenario: probe layer is hidden and attached as a sublayer
Given this probe must not affect default visuals
When the macOS source is inspected
Then it sets the probe layer hidden and attaches it with `addSublayer`
Test: `rg "setHidden: YES|addSublayer: probe_layer" platform/src/os/apple/macos/macos.rs`

### Scenario: probe tracks resize and DPI updates
Given the future split surfaces must stay aligned
When `resize_core_animation_layer` is inspected
Then it updates the probe layer `setDrawableSize`, `setContentsScale`, and `setFrame`
Test: `rg "native_interleave_probe_layer|setDrawableSize|setContentsScale|setFrame" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given the probe must not alter normal behavior
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only adds the hidden macOS layer probe
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-83-macos-interleave-layer-probe.spec platform/src/os/apple/macos/macos.rs "`
