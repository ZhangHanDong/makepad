spec: task
name: "AI Chat Liquid Glass Step 49 Interleave Guard Docs"
tags: [makepad, aichat, liquid-glass, apple-native, interleave, docs]
---

## Intent

Record the Step 48 `apple-native-interleave` backend guard in the route
decision and completion audit so the reserved name is treated as evidence of a
blocked route, not as implemented full native Liquid Glass.

## Decisions

- Document `apple-native-interleave` as a guarded unsupported value.
- Keep `AppleNativeInterleave` reserved for a future renderer split.
- Keep the completion audit conclusion as not complete.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not claim `AppleNativeInterleave` is implemented.
- Do not change the selected `ShaderBackdropInterior` route.

## Acceptance Criteria

### Scenario: route decision records guarded backend name
Given the v4.2 route decision
When backend naming is inspected
Then `apple-native-interleave` is documented as guarded and falling back to shader
Test: `rg "apple-native-interleave|Guarded backend value|falling back to shader" examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`

### Scenario: completion audit references Step 48
Given the completion audit
When current evidence is inspected
Then it references Step 48 and keeps full native interior glass incomplete
Test: `rg "aichat-liquid-glass-step-48-native-interleave-backend-guard|full native interior Liquid Glass is not implemented|Current conclusion: not complete" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: selected route is unchanged
Given Step 49 is documentation only
When the v4.2 route decision is inspected
Then it still selects `ShaderBackdropInterior`
Test: `rg "Selected Route: ShaderBackdropInterior|AppleNativeInterleave is not selected" examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`

### Scenario: documentation-only change
Given this is a docs sync
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
