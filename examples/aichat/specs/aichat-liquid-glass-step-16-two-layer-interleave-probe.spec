spec: task
name: "AI Chat Liquid Glass Step 16 Two Layer Interleave Probe"
tags: [makepad, aichat, liquid-glass, macos-native, compositing, interleave]
---

## Intent

Evaluate whether v4.2 can move from an above-Metal diagnostic overlay to a
real native interleave architecture. The probe decision is based on the
above-Metal result and the current macOS renderer shape: one Makepad
`CAMetalLayer` owns both background and foreground content.

## Decisions

- Treat the Step 15 above-Metal result as proof that AppKit glass can be
  visible over Makepad Metal in real macOS window composition.
- Do not present the Step 15 hierarchy as a usable aichat architecture because
  the native glass view sits above and covers Makepad content.
- Do not build a fake "two-layer" proof using native labels or Core Animation
  layers and call it Makepad interleave.
- A true `AppleNativeInterleave` requires at least two Makepad-rendered
  surfaces or passes: lower scene content and upper foreground text/controls.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-16-two-layer-interleave-probe.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
- `examples/aichat/specs/aichat-liquid-glass-v4.2-above-metal-probe-result.md`
- v4.2 documentation that records the route decision

### Forbidden

- Do not create a native-only text/control proof and label it Makepad
  interleave.
- Do not split the production macOS renderer in this task.
- Do not rename existing user-facing native targets to "full native".

### Out of Scope

- Implementing a multi-`CAMetalLayer` renderer.
- ShaderBackdrop implementation.
- aichat production UI migration.

## Acceptance Criteria

Scenario: interleave requirement is explicit
Test: `rg "two Makepad-rendered surfaces|AppleNativeInterleave requires" examples/aichat/specs/aichat-liquid-glass-step-16-two-layer-interleave-probe.spec examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given the above-Metal probe has visible native glass
When v4.2 evaluates the interleave route
Then the required renderer split is recorded explicitly

Scenario: above-Metal diagnostic is not promoted
Test: `rg "Do not present the Step 15 hierarchy as a usable aichat architecture|diagnostic overlay" examples/aichat/specs/aichat-liquid-glass-step-16-two-layer-interleave-probe.spec examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given the Step 15 hierarchy places native glass above Makepad content
When documenting the v4.2 route
Then the hierarchy remains a diagnostic overlay only

Scenario: fake interleave is rejected
Test: `rg "Do not create a native-only text/control proof|not present the Step 15 hierarchy" examples/aichat/specs/aichat-liquid-glass-step-16-two-layer-interleave-probe.spec`
Given native labels or Core Animation layers are available above glass
When documenting v4.2 results
Then those are rejected as evidence for Makepad foreground interleave

Scenario: route decision is updated
Test: `rg "Selected route: ShaderBackdropInterior|AppleNativeInterleave is not selected" examples/aichat/specs/aichat-liquid-glass-v4.2-route-decision.md`
Given the current renderer has one primary Makepad Metal layer
When v4.2 closes the interleave probe
Then the route decision does not choose `AppleNativeInterleave`
