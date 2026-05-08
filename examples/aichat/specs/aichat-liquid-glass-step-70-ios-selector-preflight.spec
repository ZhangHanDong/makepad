spec: task
name: "AI Chat Liquid Glass Step 70 iOS Selector Preflight"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, selector-preflight]
---

## Intent

Extend iOS native-glass runtime preflight from class presence to selector
presence. This prepares the UIKit backend implementation without instantiating
`UIGlassEffect`, `UIGlassContainerEffect`, or `UIVisualEffectView` yet.

## Decisions

- Keep Objective-C runtime preflight instead of typed iOS 26 SDK references.
- Check selectors needed by the planned underlay backend:
  `initWithEffect:`, `contentView`, `initWithStyle:`, `setTintColor:`,
  `setInteractive:`, and `setSpacing:`.
- Continue to report `PreflightFailed` until native UIKit views are actually
  created and attached.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/ios/ios.rs`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not instantiate UIKit Liquid Glass objects.
- Do not add typed UIKit 26 imports.
- Do not claim the UIKit backend is implemented.

## Acceptance Criteria

### Scenario: selector preflight checks planned backend selectors
Given iOS native glass classes are present
When selector preflight runs
Then it checks `initWithEffect:`, `contentView`, `initWithStyle:`, `setTintColor:`, `setInteractive:`, and `setSpacing:`
Test: `rg "ios_native_glass_required_selector_checks|initWithEffect|contentView|initWithStyle|setTintColor|setInteractive|setSpacing" platform/src/os/apple/ios/ios.rs`

### Scenario: selector-missing reason is explicit
Given a required selector is missing
When iOS native glass fallback logs
Then it uses `uikit-glass-selector-missing` and logs `missing_selector`
Test: `rg "uikit-glass-selector-missing|missing_selector" platform/src/os/apple/ios/ios.rs`

### Scenario: implementation remains pending after selectors pass
Given required classes and selectors are present
When fallback reason is computed
Then it remains `uikit-backend-implementation-pending`
Test: `rg "uikit-backend-implementation-pending|ios_native_glass_selector_preflight_result_from_missing_selector" platform/src/os/apple/ios/ios.rs`

### Scenario: docs keep UIKit incomplete
Given selector preflight does not create views
When API matrix and completion audit are inspected
Then they mention selector preflight and still say UIKit backend is not implemented
Test: `rg "selector preflight|UIKit backend is not implemented|not implemented" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: iOS target compiles
Given selector preflight is wired into iOS platform ops
When iOS platform is checked
Then it compiles for `aarch64-apple-ios`
Test: `cargo check -p makepad-platform --target aarch64-apple-ios --release`
