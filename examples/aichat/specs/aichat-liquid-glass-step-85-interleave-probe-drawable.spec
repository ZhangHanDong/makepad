spec: task
name: "AI Chat Liquid Glass Step 85 Interleave Probe Drawable"
tags: [makepad, liquid-glass, apple-native, interleave, macos, probe]
---

## Intent

Extend the hidden macOS interleave layer probe so it checks that the second
`CAMetalLayer` can produce a drawable. This is still a lifecycle probe only:
it must not route Makepad draw passes or render business UI into the probe
layer.

## Decisions

- Attempt `nextDrawable` only once per probe window to avoid consuming the
  probe layer drawable pool repeatedly.
- Immediately `present` the probe drawable when one is returned so it is not
  held indefinitely.
- Emit explicit success or unavailable logs for runtime validation.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos.rs`
- this step spec

### Constraints

- Do not draw Makepad content into the probe layer.
- Do not change primary window draw-pass routing.
- Do not expose `AppleNativeInterleave` as an available backend.

## Acceptance Criteria

### Scenario: probe tracks whether drawable check already ran
Given the probe must not consume drawables every frame
When the `MetalWindow` struct is inspected
Then it stores `native_interleave_probe_drawable_checked`
Test: `rg "native_interleave_probe_drawable_checked" platform/src/os/apple/macos/macos.rs`

### Scenario: probe attempts nextDrawable once
Given the hidden probe layer is installed
When resize/DPI synchronization runs
Then it calls `nextDrawable` only when the probe exists and has not already been checked
Test: `rg "native_interleave_probe_layer != nil|!self\\.native_interleave_probe_drawable_checked|nextDrawable" platform/src/os/apple/macos/macos.rs`

### Scenario: returned probe drawable is presented immediately
Given the probe drawable should not be held indefinitely
When a drawable is returned
Then the code calls `present` on that drawable
Test: `rg "probe_drawable.*present|present\\]" platform/src/os/apple/macos/macos.rs`

### Scenario: runtime logs include drawable status
Given runtime validation uses Studio logs
When the macOS source is inspected
Then success and unavailable logs for `native-interleave-layer-probe drawable` exist
Test: `rg "native-interleave-layer-probe drawable=available|native-interleave-layer-probe drawable=unavailable" platform/src/os/apple/macos/macos.rs`

### Scenario: platform crate still compiles
Given the probe must not alter normal behavior
When the platform crate is checked
Then it compiles in release mode
Test: `cargo check -p makepad-platform --release`

### Scenario: no unrelated files are changed
Given this step only extends the hidden macOS layer probe
When the worktree status is inspected
Then only macOS platform source and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-85-interleave-probe-drawable.spec platform/src/os/apple/macos/macos.rs "`
