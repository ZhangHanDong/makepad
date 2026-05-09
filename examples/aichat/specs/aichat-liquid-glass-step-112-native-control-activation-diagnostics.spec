spec: task
name: "AI Chat Liquid Glass Step 112 Native Control Activation Diagnostics"
tags: [makepad, macos, aichat, liquid-glass, native-controls, diagnostics]
---

## Intent

Add log evidence for the two halves of native control activation: AppKit
target/action delivery and Makepad Button action conversion. This makes the
manual/system-click verdict inspectable without relying only on visual state.

## Decisions

- The macOS `NativeGlassControlTarget` logs when AppKit calls
  `nativeGlassControlAction:`.
- `Button` logs when a `NativeGlassControlActivatedEvent` is converted into
  `ButtonAction::Clicked`.
- Logs include the native control id and use the existing `[liquid-glass]`
  prefix.
- The diagnostics are generic for opt-in native controls and do not make native
  controls default.

## Boundaries

- Do not change hit-test ownership.
- Do not change native control descriptor shape.
- Do not change aichat button behavior.
- Do not claim real click validation until a system/manual click produces the
  new logs.

## Out of Scope

- Automated macOS accessibility click tooling.
- macOS 26 glass button styling.
- UIKit native controls.
- Accessibility ownership.

## Acceptance Criteria

### Scenario: AppKit target action is logged

Test: `rg "event=target-action" platform/src/os/apple/macos/macos_delegates.rs`

Given a native AppKit control mirror
When AppKit invokes `nativeGlassControlAction:`
Then the macOS target logs a liquid-glass target-action diagnostic.

### Scenario: Button conversion is logged

Test: `rg "event=button-action" widgets/src/button.rs`

Given a native activation event matching an opt-in Button
When Button converts it into `ButtonAction::Clicked`
Then Button logs a liquid-glass button-action diagnostic.

### Scenario: completion audit keeps click verdict incomplete

Test: `rg "target-action|button-action|real AppKit click" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given diagnostics are present
When reviewing the completion audit
Then the audit still requires real AppKit click evidence before marking native
interactive controls complete.
