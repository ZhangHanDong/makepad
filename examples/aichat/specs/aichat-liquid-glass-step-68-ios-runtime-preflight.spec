spec: task
name: "AI Chat Liquid Glass Step 68 iOS Runtime Preflight"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, runtime-preflight]
---

## Intent

Replace the iOS native-glass fallback's single generic SDK-pending reason with
an Objective-C runtime class preflight. This keeps UIKit backend creation
disabled, but lets iOS 26 runtime validation distinguish missing Liquid Glass
classes from a present-but-unimplemented backend.

## Decisions

- Use Objective-C runtime class lookup, not typed UIKit 26 symbols.
- Check `UIVisualEffectView`, `UIGlassContainerEffect`, and `UIGlassEffect`.
- Keep `CxOsOp::SetNativeGlassBatch` in explicit fallback after preflight.
- Emit `WindowNativeSubstrateResolved` with `PreflightFailed` until the UIKit
  backend actually creates native views.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/ios/ios.rs`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not instantiate `UIGlassEffect`, `UIGlassContainerEffect`, or
  `UIVisualEffectView`.
- Do not add typed UIKit 26 imports.
- Do not claim the UIKit backend is implemented.

## Acceptance Criteria

### Scenario: required runtime class list is explicit
Given iOS native glass preflight runs
When required classes are inspected
Then it checks `UIVisualEffectView`, `UIGlassContainerEffect`, and `UIGlassEffect`
Test: `rg "ios_native_glass_required_class_names|UIVisualEffectView|UIGlassContainerEffect|UIGlassEffect" platform/src/os/apple/ios/ios.rs`

### Scenario: missing class maps to class-missing reason
Given iOS native glass preflight finds a missing class
When the fallback reason is computed
Then it uses `uikit-glass-class-missing`
Test: `rg "uikit-glass-class-missing|missing_class" platform/src/os/apple/ios/ios.rs`

### Scenario: present classes still do not claim implementation
Given all required runtime classes are present
When the fallback reason is computed
Then it uses `uikit-backend-implementation-pending`
Test: `rg "uikit-backend-implementation-pending|ios_native_glass_preflight_result_from_missing_class" platform/src/os/apple/ios/ios.rs`

### Scenario: iOS op logs runtime preflight detail
Given the iOS platform op loop receives `SetNativeGlassBatch`
When native glass fallback is inspected
Then it logs `backend=apple-native-ios` with runtime preflight details
Test: `rg "ios_native_glass_runtime_preflight|backend=apple-native-ios|missing_class" platform/src/os/apple/ios/ios.rs`

### Scenario: docs keep UIKit incomplete
Given this step only adds runtime preflight
When API matrix and completion audit are inspected
Then they still say UIKit backend is not implemented
Test: `rg "runtime class preflight|UIKit backend is not implemented|not implemented" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: platform compiles
Given iOS preflight is wired into platform ops
When platform is checked
Then it compiles
Test: `cargo check -p makepad-platform --target aarch64-apple-ios --release`
