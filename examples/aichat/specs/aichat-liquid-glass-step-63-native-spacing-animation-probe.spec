spec: task
name: "AI Chat Liquid Glass Step 63 Native Spacing Animation Probe"
tags: [makepad, liquid-glass, macos-native, spacing, morph, studio]
---

## Intent

Add a dedicated native spacing animation probe so Phase H animated
container-spacing behavior can be exercised through Studio without changing the
normal aichat native runnables.

## Decisions

- Gate the probe behind `AICHAT_NATIVE_SPACING_PROBE=animate`.
- Keep the default native runnables unchanged.
- Animate `GlassContainer.spacing` with a deterministic 12.0 -> 36.0 -> 12.0
  ping-pong curve.
- Stop the probe after 120 next-frame updates to avoid unbounded Studio log
  volume.
- Log probe frames and spacing values with a `[liquid-glass]` prefix.
- Add a dedicated Studio runnable:
  `makepad-example-aichat-macos-native-clear-spacing-probe`.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `makepad.splash`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not change normal `macos-native-clear` behavior.
- Do not create additional native panels.
- Do not rely on raw `cargo run` for runtime validation.

## Acceptance Criteria

### Scenario: spacing probe env parsing is explicit
Given `AICHAT_NATIVE_SPACING_PROBE` is set
When the value is `animate`, `1`, or `true`
Then the spacing animation probe is enabled
Test: `cargo test -p makepad-example-aichat aichat_native_spacing_probe_env_accepts_animate_values --release`

### Scenario: spacing curve is deterministic
Given the native spacing animation probe is enabled
When probe spacing is computed for frames 0, 60, and 120
Then it returns 12.0, 36.0, and 12.0
Test: `cargo test -p makepad-example-aichat aichat_native_spacing_probe_spacing_ping_pongs --release`

### Scenario: spacing probe is bounded
Given the native spacing animation probe is enabled
When probe continuation is evaluated
Then frames 0 and 119 continue and frame 120 stops
Test: `cargo test -p makepad-example-aichat aichat_native_spacing_probe_has_frame_limit --release`

### Scenario: Studio runnable exists
Given runtime validation must use Studio RunItem
When runnables are inspected
Then `makepad-example-aichat-macos-native-clear-spacing-probe` injects `AICHAT_NATIVE_SPACING_PROBE=animate`
Test: `rg "macos-native-clear-spacing-probe|AICHAT_NATIVE_SPACING_PROBE|spacing_probe" makepad.splash`

### Scenario: normal native clear runnable remains unchanged
Given the spacing probe has a dedicated runnable
When normal native clear runnables are inspected
Then `makepad-example-aichat-macos-native-clear` still exists without a spacing probe override
Test: `rg 'name: "makepad-example-aichat-macos-native-clear", backend: "macos-native-clear"' makepad.splash`

### Scenario: native spacing probe logs are discoverable
Given the spacing probe runs
When logs are inspected
Then `[liquid-glass] native-spacing-animation-probe` identifies frame and spacing values
Test: `rg "native-spacing-animation-probe|native_spacing_probe" examples/aichat/src/main.rs`

### Scenario: aichat compiles
Given spacing probe state is wired into app next-frame handling
When aichat is checked
Then it compiles
Test: `cargo check -p makepad-example-aichat --release`
