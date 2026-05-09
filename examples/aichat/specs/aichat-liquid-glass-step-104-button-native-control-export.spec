spec: task
name: "AI Chat Liquid Glass Step 104 Button Native Control Export"
tags: [makepad, widgets, button, liquid-glass, native-controls]
---

## Intent

Add an opt-in native-control descriptor export path to the base `Button` widget
so existing `GlassButton` aliases can later mirror themselves as native Apple
controls without changing their Makepad action ownership. The default remains
off, so existing buttons and aichat runtime behavior stay unchanged.

## Decisions

- Add `Button.native_control: bool`, default `false`.
- Add `Button.native_control_role`, mapped to `NativeGlassButtonRole`.
- When `native_control` is true, `Button::draw_walk` pushes a
  `NativeGlassControlDescriptor` into the `GlassContainer` collector.
- Keep the descriptor style fixed to `NativeGlassStyle::Clear` for this first
  export slice; visual style expansion belongs to a later tuning step.
- Keep `GlassButton` as the existing `ButtonFlat` alias.
- Do not create AppKit/UIKit controls in this step.

## Boundaries

### Allowed Changes

- `widgets/src/button.rs`
- `widgets/src/glass_panel.rs`
- `examples/aichat/specs/aichat-liquid-glass-step-104-button-native-control-export.spec`
- `examples/aichat/specs/APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Forbidden

- Do not refactor `GlassButton` into a new Rust widget.
- Do not turn native control export on by default.
- Do not create or install native AppKit/UIKit controls.
- Do not change button action ownership or `ButtonAction`.
- Do not change aichat runtime behavior.

## Out of Scope

- Native button action bridge.
- Native button installer.
- Accessibility bridge.
- Per-button native style/tint tuning.

## Acceptance Criteria

Scenario: button exposes opt-in native control properties
Test: `rg "native_control|native_control_role|ButtonNativeGlassRole" widgets/src/button.rs`
Given buttons remain Makepad-rendered by default
When the button widget fields are inspected
Then native control export is opt-in and has a role field

Scenario: button pushes native control descriptors when opted in
Test: `rg "push_native_glass_control_descriptor|NativeGlassControlDescriptor|NativeGlassControlKind::Button" widgets/src/button.rs widgets/src/glass_panel.rs`
Given a button has `native_control: true`
When it draws inside a native `GlassContainer`
Then it can push a native control descriptor into the collector

Scenario: native button role mapping is tested
Test: `cargo test -p makepad-widgets button_native_glass_role -- --nocapture`
Given the button role field is mapped to the shared descriptor role
When widget tests run
Then each `ButtonNativeGlassRole` maps to the expected `NativeGlassButtonRole`

Scenario: GlassButton alias remains unchanged
Test: `rg "mod.widgets.GlassButton = mod.widgets.ButtonFlat" widgets/src/glass_panel.rs`
Given the export path lives in base `Button`
When glass widget definitions are inspected
Then `GlassButton` remains the existing styled alias

Scenario: completion audit distinguishes export from native installation
Test: `rg "Step 104 adds opt-in Button descriptor export|native interactive controls are not implemented" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
Given buttons can now export native-control descriptors
When the completion audit is inspected
Then it still does not claim native interactive controls are implemented
