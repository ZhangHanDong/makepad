spec: task
name: "AI Chat Liquid Glass Step 106 Native Control Label Descriptor"
tags: [makepad, widgets, button, liquid-glass, native-controls, descriptors]
---

## Intent

Add textual content to native control descriptors so a future AppKit/UIKit
button installer can create a native button with the same visible title as the
Makepad semantic button that exported it.

## Decisions

- Add `NativeGlassControlDescriptor.label: String`.
- `Button.native_control` exports `self.text` into the descriptor label.
- Batch equivalence treats label changes as native update changes.
- Empty labels are allowed for icon-only controls; this step does not add icon
  transport.

## Boundaries

- Do not install native controls in platform backends in this step.
- Do not change `Button.text` behavior or existing Makepad rendering.
- Do not require labels for hidden or icon-only controls.
- Do not introduce platform-specific AppKit/UIKit text types into shared
  descriptors.

## Out of Scope

- Native button target/action implementation.
- Native icon/image transport.
- Native attributed titles.
- Accessibility labels beyond the visible descriptor label.

## Acceptance Criteria

### Scenario: descriptor carries a label

Test: `rg "label: String" platform/src/event/window.rs`

Given native control descriptors
When representing a button-like native control
Then the descriptor contains a platform-neutral `label: String`.

### Scenario: button exports its text as native label

Test: `rg "label: self.text.as_ref\\(\\).to_string\\(\\)" widgets/src/button.rs`

Given `Button.native_control` is true
When the button pushes `NativeGlassControlDescriptor`
Then it copies its current text into the descriptor label.

### Scenario: label changes trigger native update

Test: `cargo test -p makepad-platform native_glass_control_batch_equivalent_for_native_update_detects_changes -- --nocapture`

Given two otherwise equivalent native control batches
When a control label changes
Then `equivalent_for_native_update` returns false.

### Scenario: collector preserves labels

Test: `cargo test -p makepad-widgets native_glass_collector_returns_container_with_pushed_controls -- --nocapture`

Given the `GlassContainer` native collector
When a control descriptor with label `Send` is pushed
Then the finished collection preserves that label.
