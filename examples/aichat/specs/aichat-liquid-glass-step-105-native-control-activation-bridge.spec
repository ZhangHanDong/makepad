spec: task
name: "AI Chat Liquid Glass Step 105 Native Control Activation Bridge"
tags: [makepad, widgets, button, liquid-glass, native-controls, actions]
---

## Intent

Add the shared action bridge that lets a future AppKit/UIKit native control
activation enter the existing Makepad widget action path without requiring
application code to listen for a second native-control event model.

## Decisions

- Native platform backends will report a native control click through a shared
  `NativeGlassControlActivatedEvent`.
- `NativeGlassControlActivatedEvent.control_id` uses the same `LiveId` that
  `Button.native_control` exports in `NativeGlassControlDescriptor.id`.
- `Button.native_control` handles matching activation events by emitting the
  existing `ButtonAction::Clicked` action.
- The bridge does not create AppKit/UIKit controls yet; it only defines the
  application-facing activation path.
- Native activation uses default key modifiers for this first bridge slice.

## Boundaries

- Do not change existing Makepad mouse/touch hit handling for ordinary buttons.
- Do not require aichat or other apps to handle a new native-only click event.
- Do not install `NSButton` or `UIButton` views in this step.
- Do not implement accessibility nodes in this step.
- Keep `native_control` disabled by default.

## Out of Scope

- AppKit target/action class implementation.
- UIKit target/action implementation.
- Native hover/down/focus synchronization.
- Native button visual styling.

## Acceptance Criteria

### Scenario: shared native activation event exists

Test: `rg "pub struct NativeGlassControlActivatedEvent|window_id: WindowId|control_id: LiveId" platform/src/event/window.rs`

Given the platform window event model
When searching for `NativeGlassControlActivatedEvent`
Then `platform/src/event/window.rs` defines the event with `window_id` and
`control_id`.

### Scenario: button descriptor id and activation id share the same mapping

Test: `cargo test -p makepad-widgets button_native_glass_control_id -- --nocapture`

Given a native-control-enabled button
When converting its widget uid into a native control id
Then `Button::native_control_id_for_widget_uid` returns `LiveId(uid.0)`.

### Scenario: button handles native activation through existing action surface

Test: `rg "NativeGlassControlActivatedEvent|ButtonAction::Clicked\\(KeyModifiers::default\\(\\)\\)" widgets/src/button.rs`

Given a `Button` with `native_control: true`
When it receives a matching `NativeGlassControlActivatedEvent`
Then `Button::handle_event` emits `ButtonAction::Clicked` instead of a new
app-facing native-control action.

### Scenario: current completion audit remains honest

Test: `rg "Step 105 adds|native interactive controls are not implemented|no backend posts it yet" examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

Given this bridge does not install platform controls
When reviewing the completion audit
Then the audit still says native interactive controls are not implemented.
