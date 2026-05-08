spec: task
name: "AI Chat Liquid Glass Step 72 iOS Style Override"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, style-mapping]
---

## Intent

Make the iOS UIKit native-glass installer easier to validate on iOS 26 by
adding raw-value environment overrides for `UIGlassEffect.Style`. The default
mapping remains `regular=0` and `clear=1`, but runtime validation can quickly
test alternate values without rebuilding.

## Decisions

- Keep the default iOS style mapping explicit and tested.
- Use iOS-specific environment variables so macOS validation knobs remain
  separate:
  - `AICHAT_IOS_GLASS_STYLE_REGULAR_RAW`
  - `AICHAT_IOS_GLASS_STYLE_CLEAR_RAW`
- Use the shared `NativeGlassStyle` boundary because the same type already owns
  the macOS raw mapping.

## Boundaries

### Allowed Changes

- `platform/src/event/window.rs`
- `platform/src/os/apple/ios/ios_app.rs`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- this step spec

### Constraints

- Do not change the macOS style mapping.
- Do not claim iOS style values are verified until an iOS 26 runtime confirms
  them.
- Do not add typed UIKit 26 symbols.

## Acceptance Criteria

### Scenario: default iOS style values are stable
Given no iOS style override is set
When `NativeGlassStyle` maps to iOS raw values
Then `Regular` maps to `0` and `Clear` maps to `1`
Test: `cargo test -p makepad-platform native_glass_style_maps_to_ios_default_raw_values --release`

### Scenario: iOS regular override is accepted
Level: unit env parsing
Targets: `NativeGlassStyle::ios_raw_value_from_override`
Given `AICHAT_IOS_GLASS_STYLE_REGULAR_RAW` contains an integer
When regular style is mapped for iOS
Then that integer is used
And the override path is iOS-specific and does not mention the macOS override variables
Test: `cargo test -p makepad-platform native_glass_style_ios_raw_value_accepts_override --release`

### Scenario: invalid override falls back
Given an iOS style override contains a non-integer value
When the style is mapped for iOS
Then the default raw value is used
Test: `cargo test -p makepad-platform native_glass_style_ios_raw_value_rejects_invalid_override --release`

### Scenario: iOS override variable names are present
Given iOS style validation needs runtime knobs
When the shared style mapping is inspected
Then it contains `AICHAT_IOS_GLASS_STYLE_REGULAR_RAW` and `AICHAT_IOS_GLASS_STYLE_CLEAR_RAW`
Test: `rg "AICHAT_IOS_GLASS_STYLE_REGULAR_RAW|AICHAT_IOS_GLASS_STYLE_CLEAR_RAW" platform/src/event/window.rs examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`

### Scenario: UIKit installer uses shared mapping
Given the iOS native glass installer creates `UIGlassEffect`
When it sends `initWithStyle:`
Then it uses `NativeGlassStyle::ios_raw_value`
Test: `rg "initWithStyle: panel.style.ios_raw_value|AICHAT_IOS_GLASS_STYLE" platform/src/os/apple/ios/ios_app.rs platform/src/event/window.rs`

### Scenario: no typed UIKit 26 symbols are added
Given the local SDK may not expose typed Liquid Glass APIs
When the iOS installer and shared style mapping are inspected
Then no typed UIKit 26 glass imports or Rust types are introduced
Test: `! rg "class!\\(UIGlassEffect\\)|class!\\(UIGlassContainerEffect\\)|UIGlassEffect::|UIGlassContainerEffect::|struct UIGlassEffect|struct UIGlassContainerEffect" platform/src/os/apple/ios/ios_app.rs platform/src/event/window.rs`

### Scenario: iOS target still compiles
Level: compile-only SDK compatibility
Targets: `platform/src/os/apple/ios/ios_app.rs`, `platform/src/event/window.rs`
Given the override path uses only Rust env parsing and dynamic Objective-C calls
When iOS platform is checked
Then it compiles for `aarch64-apple-ios`
Test: `cargo check -p makepad-platform --target aarch64-apple-ios --release`
