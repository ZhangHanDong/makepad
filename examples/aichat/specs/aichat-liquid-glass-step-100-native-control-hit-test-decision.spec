spec: task
name: "AI Chat Liquid Glass Step 100 Native Control Hit-Test Decision"
tags: [makepad, aichat, liquid-glass, native-controls, hit-test, policy]
---

## Intent

Close the blocking ownership decision for future native interactive Liquid
Glass controls. The current Makepad-rendered `GlassButton` family remains the
production implementation, but the next native-control prototype now has a
specific hit-test and action ownership model.

## Decisions

- First native-control prototype uses direct AppKit/UIKit hit testing only
  inside explicit native control rects registered by Makepad.
- Native controls are opt-in mirrors of Makepad semantic controls, not a
  replacement widget tree.
- Native control actions bridge back into Makepad actions; Makepad remains the
  app state owner.
- Makepad does not synthesize low-level mouse/key events into AppKit/UIKit in
  v4.10.
- Native glass panels outside explicit native control rects stay passthrough.
- Studio widget inspection continues to report Makepad semantic owners.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
- `examples/aichat/specs/aichat-liquid-glass-step-100-native-control-hit-test-decision.spec`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Forbidden

- Do not implement native AppKit/UIKit controls in this step.
- Do not change `NativeGlassBatch::validate_v4_1`.
- Do not allow `NativeGlassHitTest::Interactive` for glass panels.
- Do not change aichat runtime behavior.

## Out of Scope

- Native `NSButton` / `UIButton` creation.
- Action bridge implementation.
- Accessibility tree implementation.
- UIKit runtime validation.

## Acceptance Criteria

Scenario: ownership decision is documented
Test: `rg "direct platform hit testing|explicit native controls|Native controls are opt-in mirrors" examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
Given native interactive controls are still future work
When the policy is inspected
Then it states that AppKit/UIKit own hit testing only for explicit native controls

Scenario: Makepad remains the app state owner
Test: `rg "Native control actions bridge back into Makepad|app state remains owned by Makepad|does not synthesize low-level mouse/key events" examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
Given native controls need to report actions
When the ownership policy is inspected
Then native actions bridge back to Makepad instead of replacing Makepad state ownership

Scenario: existing panel hit-test guard remains
Test: `rg "NativeGlassBatch::validate_v4_1|NativeGlassHitTest::Interactive|InteractiveHitTestUnsupported" platform/src/event/window.rs examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
Given v4.1 native glass panels are visual surfaces only
When the guard is inspected
Then interactive glass panels remain rejected

Scenario: completion audit distinguishes decision from implementation
Test: `rg "hit-test ownership decision landed|native interactive controls are not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
Given the ownership decision is now closed
When the completion audit is inspected
Then it still does not claim native interactive controls are implemented
