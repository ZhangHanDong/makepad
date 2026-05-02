spec: task
name: "AI Chat Liquid Glass Step 04 Widget Export"
tags: [makepad, widgets, liquid-glass, glass-panel, descriptor-export, phase-d]
---

## Intent

Teach the Makepad widget layer to opt into native Liquid Glass descriptors while
preserving the existing shader-rendered `GlassPanel` behavior by default. This
adds `GlassContainer` as the single v4.1 descriptor batch root and turns
`GlassPanel` into a wrapper widget that can both draw the existing shader panel
and export native panel metadata when requested.

## Decisions

- Keep `GlassPanel.native` defaulting to `false` so existing apps keep the
  current shader overlay behavior.
- Add `GlassContainer.native` defaulting to `false`; only a native container
  emits `CxOsOp::SetNativeGlassBatch`.
- Collect descriptors through widget tree traversal during draw, after layout
  has produced final rects.
- Use widget UID-derived `LiveId` values for container and panel identifiers.
- Preserve native hit-test as a property but default it to passthrough.
- Use `f64` for `native_z_order` in script-facing widget state and cast to
  `i32` when building the platform descriptor.

## Constraints

- Must not require existing `GlassPanel` users to set new native properties.
- Must not add AppKit/UIKit references to widgets.
- Must not emit more than one container from a single `GlassContainer` draw.
- Must keep native descriptor export opt-in.

## Boundaries

### Allowed Changes

- `examples/aichat/specs/aichat-liquid-glass-step-04-widget-export.spec`
- `examples/aichat/specs/aichat-liquid-glass-v4-phase-d.plan.md`
- `widgets/src/glass_panel.rs`

### Forbidden

- `platform/src/**`
- `examples/aichat/src/**`
- `examples/aichat/prototypes/**`

### Out of Scope

- aichat wiring and visual tuning.
- macOS installer behavior.
- iOS native backend.
- ShaderBackdrop.

## Acceptance Criteria

Scenario: GlassPanel preserves shader behavior by default
Test: `rg "pub native: bool|#\\[live\\(false\\)\\]" widgets/src/glass_panel.rs && rg "mod.widgets.GlassPanel = set_type_default" widgets/src/glass_panel.rs`
Given existing apps instantiate `GlassPanel`
When they do not set native properties
Then the widget defaults to non-native behavior and keeps the `GlassPanel` script default

Scenario: GlassContainer emits one native batch after layout
Test: `rg "GlassContainer|SetNativeGlassBatch|NativeGlassBatch|containers: vec!" widgets/src/glass_panel.rs`
Given a `GlassContainer` with `native: true`
When its draw step finishes and a current window exists
Then it emits a `SetNativeGlassBatch` operation with one container descriptor

Scenario: Widget native collector unit tests pass
Test:
  Package: makepad-widgets
  Filter: native_glass
Given the native descriptor collector receives panels during draw traversal
When the widget unit tests run
Then the collector returns a container with pushed panels and maps capsule shape correctly

Scenario: Widget crate compiles after GlassPanel wrapper conversion
Test: `cargo check -p makepad-widgets`
Given `GlassPanel` changed from a pure script alias to a Rust wrapper widget
When the widgets crate is checked
Then script registration and widget APIs compile

Scenario: Step 04 commit is limited to widget export files
Test: `git diff --name-only --cached | rg -v "^(examples/aichat/specs/aichat-liquid-glass-step-04-widget-export.spec|examples/aichat/specs/aichat-liquid-glass-v4-phase-d.plan.md|widgets/src/glass_panel.rs)$" && exit 1 || exit 0`
Given only Step 04 files are staged
When the staged file list is checked
Then platform and aichat application files are absent
