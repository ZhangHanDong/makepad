spec: task
name: "AI Chat Liquid Glass Step 64 Native Inactive Probe"
tags: [makepad, liquid-glass, macos-native, inactive-window, studio]
---

## Intent

Add an opt-in native inactive-window probe so focus transitions can be validated
with explicit logs instead of relying only on visual inspection or shader-mode
inactive tint tests.

## Decisions

- Gate the probe behind `AICHAT_NATIVE_INACTIVE_PROBE=1`.
- Keep normal native runnables unchanged.
- Log only Apple-native substrates, because shader inactive behavior already has
  existing multiplier tests.
- Log active state, native style, and multiplier.
- Add a dedicated Studio runnable:
  `makepad-example-aichat-macos-native-clear-inactive-probe`.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `makepad.splash`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not change normal `macos-native-clear` behavior.
- Do not replace system native inactive glass behavior.
- Keep complete inactive-window support marked incomplete until visual
  focus-transition validation exists.

## Acceptance Criteria

### Scenario: inactive probe env parsing is explicit
Given `AICHAT_NATIVE_INACTIVE_PROBE` is set
When the value is `1` or `true`
Then the inactive probe is enabled
Test: `cargo test -p makepad-example-aichat aichat_native_inactive_probe_env_accepts_truthy_values --release`

### Scenario: inactive probe logs only native substrate
Given inactive probe logging receives a native and shader appearance
When log lines are computed
Then native returns an active/style/multiplier line and shader returns no line
Test: `cargo test -p makepad-example-aichat aichat_native_inactive_probe_logs_only_native_substrate --release`

### Scenario: Studio runnable exists
Given runtime validation must use Studio RunItem
When runnables are inspected
Then `makepad-example-aichat-macos-native-clear-inactive-probe` injects `AICHAT_NATIVE_INACTIVE_PROBE=1`
Test: `rg "macos-native-clear-inactive-probe|AICHAT_NATIVE_INACTIVE_PROBE|inactive_probe" makepad.splash`

### Scenario: normal native clear runnable remains unchanged
Given the inactive probe has a dedicated runnable
When normal native clear runnables are inspected
Then `makepad-example-aichat-macos-native-clear` still exists without an inactive probe override
Test: `rg 'name: "makepad-example-aichat-macos-native-clear", backend: "macos-native-clear"' makepad.splash`

### Scenario: completion audit keeps inactive support incomplete
Given this step adds probe logs but not visual focus-transition validation
When completion audit status is inspected
Then native inactive remains marked as needing visual focus-transition validation
Test: `rg "native inactive has a log probe but still needs visual focus-transition validation" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: native inactive probe logs are discoverable
Given the inactive probe runs
When logs are inspected
Then `[liquid-glass] native-inactive-probe` identifies active state, style, and multiplier
Test: `rg "native-inactive-probe|native_inactive_probe" examples/aichat/src/main.rs`

### Scenario: aichat compiles
Given inactive probe state is wired into focus handling
When aichat is checked
Then it compiles
Test: `cargo check -p makepad-example-aichat --release`
