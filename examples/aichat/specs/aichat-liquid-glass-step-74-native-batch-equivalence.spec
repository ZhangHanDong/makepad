spec: task
name: "AI Chat Liquid Glass Step 74 Native Batch Equivalence"
tags: [makepad, liquid-glass, apple-native, ios, phase-g, descriptor-diff]
---

## Intent

Add a shared native-glass batch equivalence helper and use it in the iOS UIKit
installer so repeated identical descriptor batches do not remove and recreate
native UIKit views unnecessarily.

## Decisions

- Keep the equivalence helper in the shared descriptor module because the logic
  is AppKit/UIKit independent.
- Use the same subpixel tolerance already used by the macOS backend: `0.5`
  logical units.
- Treat spacing/style/tint/hit-test/z-order/visibility changes as meaningful.
- Start by wiring the helper into iOS; macOS can be migrated from its private
  equivalent helper in a later cleanup.

## Boundaries

### Allowed Changes

- `platform/src/event/window.rs`
- `platform/src/os/apple/ios/ios_app.rs`
- this step spec

### Constraints

- Do not change macOS backend behavior in this step.
- Do not change descriptor public fields.
- Do not relax validation rules.

## Acceptance Criteria

### Scenario: subpixel layout jitter is ignored
Given two native glass batches differ only by less than half a logical pixel
When shared native update equivalence is checked
Then they are considered equivalent
Test: `cargo test -p makepad-platform native_glass_batch_equivalent_for_native_update_tolerates_subpixel_jitter --release`

### Scenario: semantic changes are detected
Given two native glass batches differ in spacing, style, tint, hit-test, z-order, or visibility
When shared native update equivalence is checked
Then they are not considered equivalent
Test: `cargo test -p makepad-platform native_glass_batch_equivalent_for_native_update_detects_semantic_changes --release`

### Scenario: iOS installer reuses equivalent result
Given iOS receives a native glass batch equivalent to the last installed batch
When `install_native_glass_batch` runs
Then it returns the cached result without reinstalling UIKit views
Test: `rg "equivalent_for_native_update|last_native_glass_batch_result" platform/src/os/apple/ios/ios_app.rs platform/src/event/window.rs`

### Scenario: validation rules remain strict
Given v4.1 still rejects interactive hit testing and over-budget panel batches
When native glass validation tests run
Then those invalid batches are still rejected
Test: `cargo test -p makepad-platform native_glass_batch_rejects --release`

### Scenario: descriptor public fields are unchanged
Given this step only adds helper behavior
When the shared descriptor structs are inspected
Then `NativeGlassBatch`, `NativeGlassContainerDescriptor`, and `NativeGlassPanelDescriptor` still expose the same public descriptor fields
Test: `rg "pub struct NativeGlassBatch|pub struct NativeGlassContainerDescriptor|pub struct NativeGlassPanelDescriptor|pub window_id|pub containers|pub panels|pub rect|pub style|pub hit_test" platform/src/event/window.rs`

### Scenario: macOS backend is untouched
Given this step starts by wiring equivalence into iOS only
When the staged diff is inspected
Then no macOS backend files are changed
Test: `! git diff --name-only --cached | rg "^platform/src/os/apple/macos/"`

### Scenario: platform checks still pass
Given the helper is shared and iOS uses it
When host and iOS platform are checked
Then both compile in release mode
Test: `cargo check -p makepad-platform --release && cargo check -p makepad-platform --target aarch64-apple-ios --release`
