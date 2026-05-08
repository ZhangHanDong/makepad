spec: task
name: "AI Chat Liquid Glass Step 79 Native Interleave Request"
tags: [makepad, liquid-glass, apple-native, interleave, backend]
---

## Intent

Prepare the reserved `AppleNativeInterleave` path for future renderer-split
work by making it an explicit backend request variant instead of collapsing it
to `Shader` during environment parsing.

## Decisions

- `AICHAT_GLASS_BACKEND=apple-native-interleave` remains unsupported in this
  step.
- Parsing should preserve the requested backend as
  `GlassBackendRequest::AppleNativeInterleave` with no parse warning.
- Resolution should continue to fall back to shader and return the existing
  renderer-split warning.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- this step spec

### Constraints

- Do not expose `AppleNativeInterleave` as an installed or active backend.
- Do not change shader backdrop, macOS native underlay, or iOS native behavior.
- Do not weaken the existing reserved-backend warning text.

## Acceptance Criteria

### Scenario: parser preserves the reserved backend request
Given the user requests `apple-native-interleave`
When the glass backend environment value is parsed
Then the result is `GlassBackendRequest::AppleNativeInterleave` with no parse warning
Test: `cargo test -p makepad-example-aichat --release aichat_glass_backend_env_contract`

### Scenario: startup resolution still falls back to shader
Given the user requests `apple-native-interleave` at startup
When startup glass appearance is resolved
Then the appearance remains shader-only with no backdrop and the renderer-split warning is returned
Test: `cargo test -p makepad-example-aichat --release aichat_apple_native_interleave_backend_is_reserved`

### Scenario: normal resolution still falls back to shader
Given `AppleNativeInterleave` has no renderer split yet
When glass appearance is resolved after startup
Then it still returns shader-only appearance and the renderer-split warning
Test: `cargo test -p makepad-example-aichat --release aichat_apple_native_interleave_resolution_requires_renderer_split`

### Scenario: existing backend behavior is not changed
Given existing shader, shader-backdrop, underlay, and auto backend values
When the backend contract test is run
Then those existing values still resolve through their previous request variants
Test: `cargo test -p makepad-example-aichat --release aichat_glass_backend_env_contract`

### Scenario: unknown backend values still fall back through the parse error path
Given the user requests an unknown backend value
When the backend contract test is run
Then the unknown value still returns a parse warning and shader fallback
Level: unit
Targets: pure `parse_glass_backend` helper, no process environment I/O
Test: `cargo test -p makepad-example-aichat --release aichat_glass_backend_env_contract`

### Scenario: no unrelated files are changed
Given this step only prepares the reserved backend request shape
When the worktree status is inspected
Then only aichat main and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-79-native-interleave-request.spec examples/aichat/src/main.rs "`
