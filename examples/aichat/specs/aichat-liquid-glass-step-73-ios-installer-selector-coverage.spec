spec: task
name: "AI Chat Liquid Glass Step 73 iOS Installer Selector Coverage"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, selector-preflight]
---

## Intent

Extend iOS native-glass selector preflight so it covers the selectors used by
the Step 71 UIKit installer, not only the core Liquid Glass selectors. This
keeps the backend fail-closed on runtimes where a required UIView/CALayer
selector is unexpectedly unavailable.

## Decisions

- Keep selector checks dynamic through Objective-C runtime calls.
- Include stable UIKit/QuartzCore selectors that the installer sends before
  attaching glass views:
  `setFrame:`, `setUserInteractionEnabled:`, `insertSubview:belowSubview:`,
  `addSubview:`, `layer`, `setMasksToBounds:`, and `setCornerRadius:`.
- Preserve missing-selector fallback logging through `missing_selector`.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/ios/ios.rs`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- this step spec

### Constraints

- Do not instantiate UIKit views during preflight.
- Do not add typed iOS 26 symbols.
- Do not change macOS preflight behavior.

## Acceptance Criteria

### Scenario: installer selector preflight is complete
Given the iOS installer sends UIKit and CALayer selectors
When required selector checks are inspected
Then it includes glass selectors plus `setFrame:`, `setUserInteractionEnabled:`, `insertSubview:belowSubview:`, `addSubview:`, `layer`, `setMasksToBounds:`, and `setCornerRadius:`
Test: `rg "setFrame:|setUserInteractionEnabled:|insertSubview:belowSubview:|addSubview:|setMasksToBounds:|setCornerRadius:" platform/src/os/apple/ios/ios.rs`

### Scenario: missing selector still fails closed
Given any required installer selector is missing
When selector preflight reports the error
Then the existing `uikit-glass-selector-missing` reason and `missing_selector` field are used
Test: `rg "uikit-glass-selector-missing|missing_selector|ios_native_glass_selector_preflight_result_from_missing_selector" platform/src/os/apple/ios/ios.rs`

### Scenario: no typed UIKit 26 symbols are added
Given the local SDK may not expose typed Liquid Glass APIs
When iOS preflight is inspected
Then it uses runtime class names and `sel!` selectors rather than typed UIKit 26 Rust symbols
Test: `! rg "class!\\(UIGlassEffect\\)|class!\\(UIGlassContainerEffect\\)|UIGlassEffect::|UIGlassContainerEffect::|struct UIGlassEffect|struct UIGlassContainerEffect" platform/src/os/apple/ios/ios.rs`

### Scenario: iOS target still compiles
Given selector preflight remains dynamic
When iOS platform is checked
Then it compiles for `aarch64-apple-ios`
Test: `cargo check -p makepad-platform --target aarch64-apple-ios --release`
