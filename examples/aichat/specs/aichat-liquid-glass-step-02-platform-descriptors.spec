spec: task
name: "AI Chat Liquid Glass Step 02 Platform Descriptors"
tags: [makepad, aichat, liquid-glass, platform, descriptors, phase-b]
---

## Intent

Add the shared, platform-neutral descriptor and result model for v4.1 Apple
native Liquid Glass batches. The model lets widgets describe native glass
containers and panels without importing AppKit/UIKit types, while preserving
v4.1 limits that make compositor ordering and input routing deterministic.

## Decisions

- Define descriptors in `platform/src/event/window.rs`, because window events
  already own platform-facing window state types.
- Re-export descriptor/result types from `platform/src/lib.rs`.
- Keep all rect, radius, and spacing values in Makepad logical units.
- v4.1 allows at most one native glass container per window.
- v4.1 allows at most twelve visible native panels per window.
- `NativeGlassHitTest::Interactive` exists for forward compatibility but
  `validate_v4_1` rejects it.
- macOS `NativeGlassStyle::Regular` maps to raw value `0`; `Clear` maps to raw
  value `1` until the runtime override path proves otherwise.

## Constraints

- Must not import or expose AppKit/UIKit types in the shared descriptor API.
- Must not add OS installation behavior in this task.
- Must keep `NativeGlassBatch.containers` as `Vec` for future compatibility,
  while validation rejects more than one container in v4.1.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-02-platform-descriptors.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-phase-b.plan.md`
- `platform/src/event/window.rs`
- `platform/src/lib.rs`

### Forbidden

- `platform/src/os/**`
- `platform/src/cx_api.rs`
- `platform/src/window.rs`
- `widgets/**`
- `examples/aichat/src/**`

### Out of Scope

- Installing native views on macOS.
- Collecting descriptors from widgets.
- aichat UI wiring or visual tuning.

## Acceptance Criteria

Scenario: Native glass descriptor API validates v4.1 limits
Test:
  Package: makepad-platform
  Filter: native_glass_batch
Given a `NativeGlassBatch` with one container and twelve visible panels
When `validate_v4_1` runs
Then it accepts that batch and rejects multiple containers, too many panels, and interactive hit-test panels

Scenario: Shared API exposes no AppKit/UIKit names
Test: `! rg "AppKit|UIKit|NSView|UIView" platform/src/event/window.rs platform/src/lib.rs`
Given the shared descriptor API is platform-neutral
When the public event and re-export files are searched
Then no AppKit/UIKit view types leak into shared Rust API

Scenario: Style and shape helpers are deterministic
Test:
  Package: makepad-platform
  Filter: native_glass_style_maps_to_macos_raw_values
Given native style helpers are used by the macOS installer
When tests check regular and clear styles
Then regular maps to `0`, clear maps to `1`, and capsule radius derives from the shortest rect side

Scenario: Step 02 commit is limited to descriptor API files
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-02-platform-descriptors.spec|examples/aichat/specs/aichat-liquid-glass-v4-phase-b.plan.md|platform/src/event/window.rs|platform/src/lib.rs)$" && exit 1 || exit 0`
Given only Step 02 files are staged
When the staged file list is checked
Then OS installers, widgets, and aichat runtime files are absent
