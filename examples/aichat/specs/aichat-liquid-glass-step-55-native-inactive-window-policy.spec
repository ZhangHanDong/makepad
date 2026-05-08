spec: task
name: "AI Chat Liquid Glass Step 55 Native Inactive Window Policy"
tags: [makepad, liquid-glass, apple-native, inactive-window, phase-h]
---

## Intent

Separate aichat's inactive-window dimming policy for shader glass and Apple
native glass. Native glass may already receive system inactive material
treatment, so the app-side multiplier should be less aggressive than the shader
fallback multiplier until full native inactive-window behavior is visually
validated.

## Decisions

- Keep shader inactive dimming at `0.70`.
- Use a weaker app-side inactive multiplier for Apple native glass.
- Treat this as a readability policy, not proof of complete native inactive
  behavior.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not claim native inactive-window support is complete.
- Do not change shader inactive-window behavior.
- Do not add platform-specific AppKit/UIKit inactive hooks.

## Acceptance Criteria

### Scenario: shader inactive multiplier remains unchanged
Given the effective appearance is shader-only
When the inactive multiplier policy is evaluated for an inactive window
Then it returns the existing shader multiplier
Test: `cargo test -p makepad-example-aichat aichat_shader_inactive_multiplier_remains_legacy --release`

### Scenario: native inactive multiplier is less aggressive
Given the effective appearance is macOS native
When the inactive multiplier policy is evaluated for an inactive window
Then it returns a multiplier higher than the shader inactive multiplier and lower than active state
Test: `cargo test -p makepad-example-aichat aichat_native_inactive_multiplier_is_less_aggressive --release`

### Scenario: active windows are not dimmed
Given any glass appearance
When the inactive multiplier policy is evaluated for an active window
Then it returns `1.0`
Test: `cargo test -p makepad-example-aichat aichat_active_inactive_multiplier_is_one --release`

### Scenario: inactive policy is documented
Given native inactive-window behavior remains a Phase H gate
When the advanced behavior gates and completion audit are inspected
Then they reference the native inactive policy and still say full native inactive support is unproven
Test: `rg "native inactive|inactive-window.*unproven|NATIVE_INACTIVE_GLASS_MULTIPLIER" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
