spec: task
name: "AI Chat Liquid Glass Step 69 iOS Underlay Host View"
tags: [makepad, liquid-glass, apple-native, ios, uikit, view-hierarchy]
---

## Intent

Prepare the iOS view hierarchy for a UIKit native-glass underlay. The current
root view controller owns the `MTKView` directly, which leaves no sibling layer
below Metal for `UIVisualEffectView` panels. This step introduces a root
`UIView` host and keeps the `MTKView` as its autoresizing child.

## Decisions

- Use a plain UIKit `UIView` as the root view controller view.
- Add the existing `MTKView` as a full-size autoresizing child.
- Keep text input, selection handles, and existing interactions attached to the
  `MTKView` so current input behavior is unchanged.
- Do not instantiate native glass views in this step.

## Boundaries

### Allowed Changes

- `platform/src/os/apple/ios/ios_app.rs`
- `examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not create `UIGlassEffect` or `UIGlassContainerEffect`.
- Do not make the Metal view transparent yet.
- Do not change Makepad input routing.

## Acceptance Criteria

### Scenario: iOS root host view exists
Given iOS app startup creates UIKit views
When the root view controller is configured
Then it uses a root `UIView` host and adds the `MTKView` as a child
Test: `rg "native_glass_host_view|setView: native_glass_host_view|addSubview: mtk_view_obj" platform/src/os/apple/ios/ios_app.rs`

### Scenario: MTKView still keeps existing child input views
Given the host view contains the MTKView
When input helper views are attached
Then text input and selection handles are still added to `mtk_view_obj`
Test: `rg "mtk_view_obj, addSubview: text_input_view|mtk_view_obj, addSubview: selection_handle_start|mtk_view_obj, addSubview: selection_handle_end" platform/src/os/apple/ios/ios_app.rs`

### Scenario: docs mark UIKit underlay host as prepared only
Given UIKit backend remains incomplete
When API matrix and completion audit are inspected
Then they mention the underlay host view but still say UIKit backend is not implemented
Test: `rg "underlay host view|UIKit backend is not implemented|not implemented" examples/aichat/specs/APPLE-LIQUID-GLASS-API-MATRIX.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: iOS target compiles
Given the iOS launch view hierarchy changed
When iOS platform is checked
Then it compiles for `aarch64-apple-ios`
Test: `cargo check -p makepad-platform --target aarch64-apple-ios --release`
