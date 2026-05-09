spec: task
name: "AI Chat Liquid Glass Step 99 Interleave Fallback to ShaderBackdrop"
tags: [makepad, aichat, liquid-glass, apple-native, interleave, shader-backdrop]
---

## Intent

Align the guarded `apple-native-interleave` backend behavior with the Step 98
manual fail verdict. Since the current native interleave prototype shows no
recognizable native transparency/refraction/liquid distortion, requests for
`apple-native-interleave` should fall back to the current complete interior
route, `ShaderBackdropInterior`, instead of plain shader-only rendering.

## Decisions

- Keep parsing `AICHAT_GLASS_BACKEND=apple-native-interleave` as a known value.
- Resolve it to `GlassSubstrate::ShaderOnly` with
  `ShaderBackdropProof::Interior`.
- Emit a warning that names the Step 98 fail verdict and
  `ShaderBackdropInterior` fallback.
- Do not expose a production `AppleNativeInterleave` runnable.

## Constraints

- Must use test-first implementation.
- Must not touch platform code.
- Must not change `macos-native` or `macos-native-clear` behavior.
- Must not change shader-backdrop proof profiles.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/aichat-liquid-glass-step-99-interleave-fallback-to-shader-backdrop.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`

### Forbidden

- `platform/**`
- `widgets/**`
- `makepad.splash`

## Acceptance Criteria

Scenario: apple-native-interleave falls back to ShaderBackdropInterior
Test: `cargo test -p makepad-example-aichat aichat_apple_native_interleave -- --nocapture`
Given `AICHAT_GLASS_BACKEND=apple-native-interleave`
When aichat resolves the backend
Then the appearance uses `ShaderBackdropProof::Interior`
And the warning mentions the Step 98 fail verdict and ShaderBackdropInterior fallback

Scenario: route decision documents the updated fallback
Test: `rg "apple-native-interleave.*Step 98|ShaderBackdropInterior fallback|fail verdict" examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given the user-visible backend guard changed
When route documentation is inspected
Then the Step 98 fail verdict and ShaderBackdropInterior fallback are documented

Scenario: no platform or runnable files are changed
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-99-interleave-fallback-to-shader-backdrop.spec examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md examples/aichat/src/main.rs "`
Level: repository state
Given this is an aichat guard behavior change
When repository status is inspected
Then only aichat code and docs are changed
And no files under `platform/**` are changed
And `makepad.splash` is not changed

Scenario: production runnable remains unexposed
Test: `! rg "RunAichat.*apple-native-interleave|full-native|apple-native-full" makepad.splash examples/aichat/src/main.rs`
Level: repository state
Given AppleNativeInterleave is still prototype-only
When runnables and backend strings are inspected
Then no production AppleNativeInterleave runnable is exposed
