spec: task
name: "AI Chat Liquid Glass Step 48 Native Interleave Backend Guard"
tags: [makepad, aichat, liquid-glass, apple-native, interleave]
---

## Intent

Reserve the `apple-native-interleave` backend name without allowing aichat to
overclaim full native Liquid Glass. Until a renderer split exists, the value
must fall back to shader with a specific warning.

## Decisions

- Accept `AICHAT_GLASS_BACKEND=apple-native-interleave` as a known but
  unsupported backend value.
- Resolve it to `ShaderOnly`.
- Emit a specific warning that names the missing renderer split.
- Keep `apple-native-underlay` as the only active Apple native backend.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- this step spec

### Constraints

- Do not add platform backend code.
- Do not expose a user-facing `full-native` backend or runnable target.
- Do not change shader-backdrop or underlay behavior.

## Acceptance Criteria

### Scenario: native interleave backend is guarded
Given a user requests `apple-native-interleave`
When backend parsing runs
Then it falls back to shader
And returns a warning that mentions `AppleNativeInterleave`
Test: `rg "apple-native-interleave|AppleNativeInterleave requires renderer split; falling back to shader" examples/aichat/src/main.rs`

### Scenario: startup resolution does not wait for unsupported interleave
Given startup resolution only waits for supported native underlay results
When `apple-native-interleave` is requested
Then it resolves immediately to `ShaderOnly` with a warning
Test: `rg "aichat_apple_native_interleave_backend_is_reserved" examples/aichat/src/main.rs`

### Scenario: existing native underlay names still parse
Given the current native underlay backend is supported on macOS
When backend parsing is tested
Then `apple-native-underlay` and `apple-native-underlay-clear` still map to `MacosNative`
Test: `rg "apple-native-underlay|apple-native-underlay-clear" examples/aichat/src/main.rs`

### Scenario: full-native target is not exposed
Given AppleNativeInterleave is reserved but unsupported
When runnable names and backend strings are inspected
Then there is no `full-native` backend or runnable target
Test: `! rg "full-native|apple-native-full|native-full" examples/aichat/src/main.rs examples/aichat/Cargo.toml`

### Scenario: aichat tests pass
Given backend parsing changed
When aichat tests run
Then the aichat crate tests pass
Test: `cargo test -p makepad-example-aichat aichat_ -- --nocapture`
