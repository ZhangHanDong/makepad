spec: task
name: "AI Chat Liquid Glass Step 75 macOS Shared Batch Equivalence"
tags: [makepad, liquid-glass, apple-native, macos, descriptor-diff]
---

## Intent

Migrate the macOS native glass backend from its private batch equivalence
helpers to the shared `NativeGlassBatch::equivalent_for_native_update` helper
introduced in Step 74. This keeps macOS and iOS descriptor diff semantics from
drifting.

## Decisions

- Preserve the existing macOS behavior: equivalent batches return the cached
  install result and avoid view churn.
- Keep the macOS subpixel and spacing regression tests, but point them at the
  shared helper.
- Do not change native AppKit installation, selector lookup, or view hierarchy.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/macos/macos_window.rs`
- this step spec

### Constraints

- Do not change macOS native view install order.
- Do not change iOS code in this step.
- Do not change descriptor public fields.

## Acceptance Criteria

### Scenario: macOS uses shared equivalence helper
Given macOS receives a batch equivalent to the last installed batch
When `install_native_glass_batch` checks for reuse
Then it calls `equivalent_for_native_update`
Test: `rg "equivalent_for_native_update" platform/src/os/apple/macos/macos_window.rs`

### Scenario: private macOS equivalence helpers are removed
Given equivalence now lives in shared descriptors
When the macOS backend is inspected
Then private `native_glass_batch_equivalent`, `native_glass_panel_equivalent`, `native_glass_rect_equivalent`, and `native_glass_tint_equivalent` helpers are gone
Test: `! rg "fn native_glass_(batch|panel|rect|tint)_equivalent\\(" platform/src/os/apple/macos/macos_window.rs`

### Scenario: macOS subpixel behavior is preserved
Given macOS regression tests cover subpixel jitter
When the test runs
Then equivalent batches still tolerate subpixel jitter and detect larger changes
Test: `cargo test -p makepad-platform native_glass_batch_equivalent_tolerates_subpixel_jitter --release`

### Scenario: macOS spacing changes are still detected
Given macOS regression tests cover container spacing
When the test runs
Then spacing changes still force a native update
Test: `cargo test -p makepad-platform native_glass_batch_equivalent_detects_spacing_change --release`

### Scenario: platform checks still pass
Given macOS uses shared descriptor equivalence
When platform is checked
Then release host compile passes
Test: `cargo check -p makepad-platform --release`
