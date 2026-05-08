spec: task
name: "AI Chat Liquid Glass Step 51 Native Interactive Controls Policy"
tags: [makepad, liquid-glass, apple-native, controls, hit-test, docs]
---

## Intent

Define the policy for future native interactive Liquid Glass controls before
any AppKit/UIKit control host is implemented. The current descriptor API exposes
`NativeGlassHitTest::Interactive` for forward compatibility, but v4.1 rejects it
and Makepad remains the input owner.

## Decisions

- Keep `NativeGlassHitTest::Interactive` rejected in the v4.1 descriptor backend.
- Treat native interactive controls as a separate phase from native glass panels.
- Require an event ownership and forwarding design before enabling native
  AppKit/UIKit controls.
- Keep Makepad-rendered `GlassButton` variants as the current production path.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not edit source code.
- Do not enable `NativeGlassHitTest::Interactive`.
- Do not claim native buttons are implemented.

## Acceptance Criteria

### Scenario: policy records current rejection evidence
Given native interactive controls are future work
When the policy is inspected
Then it references `NativeGlassHitTest::Interactive`, `validate_v4_1`, and `InteractiveHitTestUnsupported`
Test: `rg "NativeGlassHitTest::Interactive|validate_v4_1|InteractiveHitTestUnsupported" examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`

### Scenario: policy defines future gates
Given native controls need event ownership
When the policy is inspected
Then it lists hit-test, event forwarding, accessibility, Studio, and fallback gates
Test: `rg "Hit-test gate|Event forwarding gate|Accessibility gate|Studio gate|Fallback gate" examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`

### Scenario: source still rejects interactive descriptors
Given this policy does not enable interactive native controls
When the source is inspected
Then `validate_v4_1` still rejects `NativeGlassHitTest::Interactive`
Test: `rg "if panel\\.hit_test == NativeGlassHitTest::Interactive|InteractiveHitTestUnsupported" platform/src/event/window.rs`

### Scenario: completion audit references policy
Given native interactive controls are incomplete
When the completion audit is inspected
Then it references the native interactive controls policy
Test: `rg "APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: documentation-only change
Given this is a policy step
When the diff is inspected
Then no source file changed
Test: `git diff --name-only HEAD | rg -v "^examples/aichat/specs/|^\\.makepad/"`
