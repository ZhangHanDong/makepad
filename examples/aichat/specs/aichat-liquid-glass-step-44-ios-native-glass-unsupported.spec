spec: task
name: "AI Chat Liquid Glass Step 44 iOS Unsupported Reporting"
tags: [makepad, liquid-glass, apple-native, ios, phase-g]
---

## Intent

Start Phase G safely by making iOS handle `CxOsOp::SetNativeGlassBatch`
explicitly while the real UIKit Liquid Glass backend is still pending target SDK
validation. The operation should report a deterministic fallback instead of
falling through the generic unimplemented platform-op branch.

## Decisions

- Do not implement `UIGlassEffect` or `UIGlassContainerEffect` in this step.
- Emit `WindowNativeSubstrateResolved` with `PreflightFailed`, no style, and an
  iOS-specific reason.
- Log a `liquid-glass` line that names the iOS backend as unsupported.
- Leave macOS behavior unchanged.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/ios/ios.rs`
- this step spec

### Constraints

- Do not edit the shared descriptor structs.
- Do not edit macOS native glass backend code.
- Do not add UIKit selectors whose SDK availability has not been verified.
- Do not claim Phase G is implemented.

## Acceptance Criteria

### Scenario: iOS handles native glass batches explicitly
Given the iOS platform op loop
When it receives `CxOsOp::SetNativeGlassBatch`
Then it logs an iOS unsupported native glass backend line
And it emits `WindowNativeSubstrateResolved` with `PreflightFailed`
Test: `rg "CxOsOp::SetNativeGlassBatch\\(batch\\)|backend=apple-native-ios state=Unsupported|WindowNativeSubstrateState::PreflightFailed" platform/src/os/apple/ios/ios.rs`

### Scenario: no UIKit API is guessed
Given target SDK API names are still pending validation
When the iOS implementation is inspected
Then it does not mention `UIGlassEffect` or `UIGlassContainerEffect`
Test: `! rg "UIGlassEffect|UIGlassContainerEffect" platform/src/os/apple/ios/ios.rs`

### Scenario: macOS backend is untouched
Given this is an iOS reporting step
When the diff is inspected
Then macOS native glass files are unchanged
Test: `git diff --name-only HEAD -- platform/src/os/apple/macos`

### Scenario: platform compiles
Given the iOS branch imports native substrate event types
When compilation is run
Then the platform crate compiles
Test: `cargo check -p makepad-platform`
