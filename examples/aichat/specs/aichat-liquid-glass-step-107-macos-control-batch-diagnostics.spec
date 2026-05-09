spec: task
name: "AI Chat Liquid Glass Step 107 macOS Control Batch Diagnostics"
tags: [makepad, macos, liquid-glass, native-controls, diagnostics]
---

## Intent

Make the macOS backend consume `SetNativeGlassControlBatch` enough to validate,
cache, and log control batches instead of silently ignoring them. This creates a
runtime diagnostic checkpoint before AppKit `NSButton` installation is added.

## Decisions

- macOS handles `CxOsOp::SetNativeGlassControlBatch` by forwarding it to
  `MacosWindow::update_native_glass_control_batch`.
- The macOS handler validates the shared v4.10 batch rules.
- Equivalent batches are cached and skipped to avoid repeated logs.
- Valid batches log `state=Unsupported reason=installer-not-implemented`.
- Invalid batches log `state=Rejected` with a stable validation reason.

## Boundaries

- Do not create `NSButton` or other native controls in this step.
- Do not post `NativeGlassControlActivatedEvent` from macOS in this step.
- Do not change non-macOS backends.
- Do not claim native interactive controls are complete.

## Out of Scope

- AppKit target/action class implementation.
- Native button visual styling.
- aichat opt-in native buttons.
- UIKit control batch diagnostics.

## Acceptance Criteria

### Scenario: macOS no longer silently ignores control batches

Test: `rg "SetNativeGlassControlBatch\\(batch\\)|update_native_glass_control_batch" platform/src/os/apple/macos/macos.rs platform/src/os/apple/macos/macos_window.rs`

Given a queued `SetNativeGlassControlBatch`
When the macOS platform op loop processes it
Then it forwards the batch to `MacosWindow::update_native_glass_control_batch`.

### Scenario: valid batches log unsupported installer state

Test: `rg "backend=apple-native-controls|installer-not-implemented" platform/src/os/apple/macos/macos_window.rs`

Given a valid native control batch
When macOS processes it before AppKit installation exists
Then the backend logs `state=Unsupported reason=installer-not-implemented`.

### Scenario: invalid batches produce stable rejection reasons

Test: `cargo test -p makepad-platform native_glass_control_validation_reason_maps_errors -- --nocapture`

Given invalid native control batches
When macOS maps validation errors to log reasons
Then each shared validation error has a stable reason string.

### Scenario: completion audit remains honest

Test: `rg "Step 107 adds macOS validation/logging|native interactive controls are not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given macOS only validates and logs control batches
When reviewing the completion audit
Then the audit still says native interactive controls are not implemented.
