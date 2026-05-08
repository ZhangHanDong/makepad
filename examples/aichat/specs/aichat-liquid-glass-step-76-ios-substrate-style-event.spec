spec: task
name: "AI Chat Liquid Glass Step 76 iOS Substrate Style Event"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, state-reporting]
---

## Intent

Report iOS native glass style through `WindowNativeSubstrateResolvedEvent`
instead of returning `style: None` after the UIKit installer succeeds. This
keeps state reporting symmetric enough for upper layers to distinguish
regular/clear Apple native glass on iOS.

## Decisions

- Add iOS-specific variants to `WindowNativeSubstrateStyle`.
- Map shared `NativeGlassStyle::{Regular, Clear}` to those variants for iOS.
- Keep macOS event variants and behavior unchanged.

## Boundaries

### Allowed Changes

- `platform/src/event/window.rs`
- `platform/src/os/apple/ios/ios_app.rs`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not change macOS event style variants.
- Do not make aichat treat iOS runtime validation as complete.
- Do not add typed UIKit 26 symbols.

## Acceptance Criteria

### Scenario: iOS event style mapping exists
Given shared native glass styles
When iOS event styles are requested
Then regular maps to `IosGlassRegular` and clear maps to `IosGlassClear`
Test: `cargo test -p makepad-platform native_glass_style_maps_to_ios_event_styles --release`

### Scenario: iOS installer reports style
Given the iOS UIKit installer creates a compat event
When a visible passthrough panel determines the first style
Then the event contains `Some(style.ios_event_style())`
Test: `rg "style: Some\\(style\\.ios_event_style\\(\\)\\)" platform/src/os/apple/ios/ios_app.rs`

### Scenario: macOS variants remain available
Given existing macOS native glass state reporting
When shared event style definitions are inspected
Then `MacosGlassRegular` and `MacosGlassClear` are still present
Test: `rg "MacosGlassRegular|MacosGlassClear|IosGlassRegular|IosGlassClear" platform/src/event/window.rs`

### Scenario: completion audit remains honest
Given iOS style state reporting is not visual runtime validation
When the completion audit is inspected
Then it still says UIKit backend is not runtime-validated beyond the installer skeleton
Test: `rg "not runtime-validated beyond the dynamic installer skeleton" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: platform checks still pass
Given iOS state reporting uses shared Rust event enums
When host and iOS platform are checked
Then both compile in release mode
Test: `cargo check -p makepad-platform --release && cargo check -p makepad-platform --target aarch64-apple-ios --release`
