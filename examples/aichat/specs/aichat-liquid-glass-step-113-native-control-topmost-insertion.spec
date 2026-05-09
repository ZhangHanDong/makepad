spec: task
name: "AI Chat Liquid Glass Step 113 Native Control Topmost Insertion"
tags: [makepad, macos, liquid-glass, native-controls, hit-test]
---

## Intent

Place macOS native glass controls at the top of the AppKit container view
instead of relative to the Metal view, so explicit native controls have the best
chance to receive AppKit hit testing.

## Decisions

- Native controls remain separate from visual glass panels.
- Native controls are inserted above all sibling views in the window container.
- Ordinary native glass panels remain below the Metal view.
- This step does not mark click validation complete until target/action logs are
  observed.

## Boundaries

- Do not move `NSGlassEffectView` panels above Metal.
- Do not change `NativeGlassHitTest::Interactive` policy for panels.
- Do not make aichat native controls default.
- Do not change control descriptor coordinates.

## Out of Scope

- macOS 26 glass button styling.
- UIKit controls.
- Accessibility ownership.
- Multi-container native controls.

## Acceptance Criteria

### Scenario: native controls insert at container top

Test: `rg "relativeTo: nil" platform/src/os/apple/macos/macos_window.rs`

Given a macOS native control view
When installing it into the AppKit hierarchy
Then it is inserted above all sibling views in the container.

### Scenario: visual panels stay under Metal

Test: `rg "addSubview: native_container|positioned: -1i64" platform/src/os/apple/macos/macos_window.rs`

Given ordinary native glass panels
When installing the AppleNativeUnderlay backend
Then visual panels continue to use the under-Metal native container path.

### Scenario: audit still requires click logs

Test: `rg "target-action|button-action|real AppKit click" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given native controls are now inserted topmost
When reviewing completion state
Then the audit still requires real AppKit target/action and Button conversion
logs before native interactive controls are complete.
