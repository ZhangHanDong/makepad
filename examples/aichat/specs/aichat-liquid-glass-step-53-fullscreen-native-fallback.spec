spec: task
name: "AI Chat Liquid Glass Step 53 Fullscreen Native Fallback"
tags: [makepad, liquid-glass, apple-native, fullscreen, fallback]
---

## Intent

Make fullscreen behavior explicit for the current macOS native underlay path.
Until native fullscreen glass is validated, entering fullscreen suppresses the
native underlay and uses the shader path; exiting fullscreen restores the
previous native appearance.

## Decisions

- Treat fullscreen native glass as unsupported until a dedicated validation run
  proves panel alignment and native view lifetime across enter/exit.
- Do not silently leave `MacosNative` active when the main window enters
  fullscreen.
- Restore the previous native appearance after leaving fullscreen.
- Keep this policy inside aichat until the platform backend has a general
  fullscreen-native contract.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not implement full native fullscreen support.
- Do not change descriptor validation.
- Do not change the non-fullscreen native underlay path.

## Acceptance Criteria

### Scenario: fullscreen policy suppresses native appearance
Given the current glass appearance is macOS native
When a fullscreen window geometry event is handled
Then the effective glass appearance becomes shader-only and the prior native appearance is stored for restore
Test: `cargo test -p makepad-example-aichat aichat_native_fullscreen_fallback_suppresses_native --release`

### Scenario: fullscreen exit restores native appearance
Given fullscreen previously suppressed a native appearance
When a non-fullscreen window geometry event is handled
Then the saved native appearance is restored and the restore slot is cleared
Test: `cargo test -p makepad-example-aichat aichat_native_fullscreen_fallback_restores_native --release`

### Scenario: shader appearance is not affected
Given the current glass appearance is shader-only
When fullscreen changes
Then no native restore state is created
Test: `cargo test -p makepad-example-aichat aichat_native_fullscreen_fallback_ignores_shader --release`

### Scenario: fullscreen fallback is documented
Given Phase H fullscreen support is not complete
When the advanced behavior gates and completion audit are inspected
Then they document fullscreen as explicit native fallback, not completed native support
Test: `rg "fullscreen.*fallback|fullscreen.*suppressed|not full native fullscreen support" examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
