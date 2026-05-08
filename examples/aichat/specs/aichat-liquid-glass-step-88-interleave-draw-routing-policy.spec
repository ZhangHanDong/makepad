spec: task
name: "AI Chat Liquid Glass Step 88 Interleave Draw Routing Policy"
tags: [makepad, liquid-glass, apple-native, interleave, routing, policy]
---

## Intent

Define the first aichat draw-routing policy for `AppleNativeInterleave` before
any real lower/upper pass split is implemented. The policy must identify which
content may move into `LowerScene`, which content must remain in `UpperUi`, and
what evidence is required before the backend can become user-facing.

## Decisions

- `LowerScene` starts as non-interactive scene/background/proof content only.
- All text, input, controls, Markdown, generated UI, and hit-testable widgets
  remain in `UpperUi`.
- Native Apple glass is positioned between `LowerScene` and `UpperUi`; it must
  not own input in this phase.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`
- this step spec

### Constraints

- Do not change Rust code in this step.
- Do not expose `AppleNativeInterleave` as an available backend.
- Do not claim lower/upper draw routing is implemented.

## Acceptance Criteria

### Scenario: policy defines the three-layer stack
Given AppleNativeInterleave needs a concrete compositing model
When the policy is inspected
Then it defines `LowerScene`, Apple native glass, and `UpperUi` in order
Test: `rg "LowerScene|Apple native glass|UpperUi" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`

### Scenario: LowerScene starts with non-interactive content only
Given lower content will be sampled by native glass
When the policy is inspected
Then it limits `LowerScene` to non-interactive scene, background, and proof content
Test: `rg "non-interactive|scene/background/proof|must not receive input" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`

### Scenario: UpperUi keeps text and controls
Given aichat input and readability must remain Makepad-owned
When the policy is inspected
Then it keeps text, input, controls, Markdown, generated UI, and hit-testable widgets in `UpperUi`
Test: `rg "text|input|controls|Markdown|generated UI|hit-testable widgets|UpperUi" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`

### Scenario: native glass remains passthrough
Given native glass must not steal events in this phase
When the policy is inspected
Then it states native glass remains passthrough and input-owned by Makepad
Test: `rg "passthrough|Makepad-owned input|must not own input" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`

### Scenario: exposure gates remain explicit
Given AppleNativeInterleave is not ready for users
When the policy is inspected
Then it requires visual, input, Studio screenshot, resize/DPI, and lifecycle evidence before exposure
Test: `rg "Visual gate|Input gate|Studio screenshot gate|Resize/DPI gate|Lifecycle gate" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`

### Scenario: policy does not claim implementation
Given this step is documentation only
When the policy is inspected
Then it says lower/upper draw routing is not implemented yet
Test: `rg "lower/upper draw routing is not implemented" examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md`

### Scenario: no unrelated files are changed
Given this step only adds the draw-routing policy
When the worktree status is inspected
Then only the policy document and this spec are changed, ignoring `.makepad/`
Test: `test "$(git status --short --untracked-files=all | rg -v '^\\?\\? \\.makepad/' | sed 's/^...//' | sort | tr '\n' ' ')" = "examples/aichat/specs/aichat-liquid-glass-step-88-interleave-draw-routing-policy.spec examples/aichat/specs/APPLE-NATIVE-INTERLEAVE-DRAW-ROUTING-POLICY.md "`
