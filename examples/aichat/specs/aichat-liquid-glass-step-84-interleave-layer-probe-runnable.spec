spec: task
name: "AI Chat Liquid Glass Step 84 Interleave Layer Probe Runnable"
tags: [makepad, liquid-glass, apple-native, interleave, studio]
---

## Intent

Expose the hidden macOS interleave layer probe through a dedicated Studio
RunItem so runtime validation can use the remote Studio protocol instead of
ad-hoc shell launches.

## Decisions

- Add an `interleave_layer_probe` field to the existing `RunAichatMacosGlass`
  runnable template.
- Inject `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE` only from the dedicated probe
  runnable.
- Use native clear underlay for the probe so it exercises the current native
  macOS path while keeping the interleave layer hidden.

## Boundaries

### Allowed Changes

- `makepad.splash`
- this step spec

### Constraints

- Do not rename existing runnable items.
- Do not enable the interleave layer probe on normal native runnables.
- Do not launch UI apps outside Studio `RunItem`.

## Acceptance Criteria

### Scenario: runnable template injects the interleave probe env
Given the Studio runnable template owns aichat glass env overrides
When `makepad.splash` is inspected
Then it includes `interleave_layer_probe` and injects `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE`
Test: `rg "interleave_layer_probe|AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE" makepad.splash`

### Scenario: dedicated interleave layer probe runnable exists
Given runtime validation must use Studio RunItem
When runnable items are inspected
Then `makepad-example-aichat-macos-native-clear-interleave-layer-probe` exists with `interleave_layer_probe: "1"`
Test: `rg 'makepad-example-aichat-macos-native-clear-interleave-layer-probe|interleave_layer_probe: "1"' makepad.splash`

### Scenario: normal native clear runnable remains unchanged
Given the probe must be opt-in
When normal native clear runnable definitions are inspected
Then `makepad-example-aichat-macos-native-clear` still exists without an inline interleave probe override
Test: `rg 'name: "makepad-example-aichat-macos-native-clear", backend: "macos-native-clear"\\}' makepad.splash`

### Scenario: no unrelated files are changed
Given this step only exposes a Studio runnable
When the worktree status is inspected
Then only `makepad.splash` and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-84-interleave-layer-probe-runnable.spec makepad.splash "`
