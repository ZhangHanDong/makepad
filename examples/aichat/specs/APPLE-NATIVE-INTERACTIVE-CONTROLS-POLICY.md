# Apple Native Interactive Controls Policy

## Status

Policy for a later native-control phase. This document does not implement
native AppKit/UIKit controls.

## Current Evidence

The shared descriptor API contains `NativeGlassHitTest::Interactive` for forward
compatibility. It is intentionally rejected in v4.1:

- `platform/src/event/window.rs` defines `NativeGlassHitTest::Interactive`.
- `NativeGlassBatch::validate_v4_1` rejects interactive panels.
- The rejection error is `InteractiveHitTestUnsupported`.
- `widgets/src/glass_panel.rs` exposes `native_hit_test`, defaulting to
  passthrough.

Step 101 adds a separate shared descriptor model for future native interactive
controls:

- `NativeGlassControlBatch`
- `NativeGlassControlDescriptor`
- `NativeGlassControlKind`
- `NativeGlassButtonRole`
- `NativeGlassControlBatch::validate_v4_10`

These descriptors are not installed by any platform backend yet. They exist so
the native-control phase can proceed without reusing interactive glass panels
as controls.

Step 102 adds the platform op transport for those descriptors:

- `CxOsOp::SetNativeGlassControlBatch`
- `WindowHandle::set_native_glass_control_batch`

Current OS backends explicitly ignore this op. Native AppKit/UIKit controls are
still not created until the later installer and action bridge steps.

Step 103 extends `GlassContainer` collector plumbing so future widgets can push
`NativeGlassControlDescriptor` values alongside native glass panels. The current
`GlassButton` aliases still do not export descriptors.

Step 104 adds an opt-in export path to the base `Button` widget:

- `native_control: bool`, default `false`
- `native_control_role: ButtonNativeGlassRole`

Because `GlassButton` remains a `ButtonFlat` alias, it inherits this opt-in
field without changing existing action ownership. Current OS backends still
ignore native control batches.

Step 105 adds the shared activation bridge:

- `NativeGlassControlActivatedEvent { window_id, control_id }`
- `Button.native_control` converts matching activation events into the existing
  `ButtonAction::Clicked` path

This keeps application code on the current `button.clicked(actions)` API once a
future AppKit/UIKit installer starts posting native activation events.

This keeps the landed AppleNativeUnderlay path safe: AppKit native glass panels
do not become input owners, and Makepad continues to handle text, scroll,
clicks, command menus, drag, generated Splash UI, and Studio inspection.

## Current Production Policy

- `GlassButton`, `GlassIconButton`, `GlassPrimaryButton`, `GlassNavButton`, and
  `GlassUtilityButton` are Makepad-rendered controls.
- Native glass panels are visual surfaces only.
- `NativeGlassHitTest::Interactive` must stay rejected until the gates below are
  implemented and validated.
- `NativeGlassControlBatch` is the future native-control input model; it is
  separate from `NativeGlassBatch` and has no installer in production yet.
- `SetNativeGlassControlBatch` may be queued by future collectors, but current
  OS backends treat it as a no-op.
- `GlassContainer` can carry native-control descriptors once a future
  `GlassButton` implementation pushes them, but current `GlassButton` variants
  remain Makepad-rendered only.
- `Button.native_control` can export descriptors for future native button
  mirrors, but it is disabled by default and has no installer yet.
- `NativeGlassControlActivatedEvent` is the only app-facing native control
  activation bridge for the first button slice; widgets translate it back into
  their existing action APIs.
- Native button research lives in `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; it is
  not implementation evidence.

## v4.10 Ownership Decisions

The first native interactive-control prototype uses direct platform hit testing
for explicit native controls only:

- Native controls are opt-in mirrors of Makepad semantic controls, not a new
  replacement widget tree.
- AppKit/UIKit owns hit testing only inside the native control rects registered
  by Makepad.
- Native glass panels outside those explicit control rects stay passthrough.
- Native control actions bridge back into Makepad as Makepad actions; app state
  remains owned by Makepad.
- Makepad does not synthesize low-level mouse/key events into AppKit/UIKit for
  v4.10. If a platform control cannot own its own native event handling, it
  falls back to the Makepad-rendered `GlassButton` variant.
- Studio `WidgetTreeDump` / `WidgetQuery` continue to report the Makepad
  semantic owner. Native controls are platform mirrors and must log their
  mapping to the Makepad widget/control id for diagnosis.
- Accessibility ownership is platform-control first for native mirrored
  controls, with Makepad retaining ownership for non-native controls and all
  non-interactive glass panels. A later audit must prevent duplicate labels or
  conflicting focus order before this becomes production default.

This decision keeps the first native-control slice bounded: it avoids inventing
a synthetic event-forwarding layer while still allowing native buttons to use
their real platform glass appearance and accessibility behavior.

## Future Gates

### Hit-test gate

- Implement the v4.10 decision: AppKit/UIKit directly own native-control hit
  testing only for explicit native control rects.
- Define coordinate conversion between Makepad logical coordinates and native
  control bounds.
- Prove native glass panels outside explicit native controls remain
  passthrough.

### Event forwarding gate

- Define click, hover, focus, keyboard activation, text input, scroll, drag, and
  menu behavior.
- Define how native-control actions become Makepad `Actions`.
- Ensure command routing does not bypass Makepad widget state.

### Accessibility gate

- Define whether accessibility nodes come from Makepad, AppKit/UIKit, or a
  combined tree.
- Avoid duplicate labels or conflicting focus order.
- Respect Reduce Transparency and increased contrast without forcing shader
  fallback on Apple native surfaces.

### Studio gate

- `WidgetTreeDump` and `WidgetQuery` must still identify Makepad ownership for
  mixed native/Makepad controls.
- Studio screenshots must document whether native controls appear in the
  framebuffer or only in system screenshots.
- Remote clicks and typing need deterministic ownership when a native control is
  present.

### Fallback gate

- If native control classes/selectors are unavailable, controls must fall back
  to Makepad-rendered `GlassButton` variants.
- The fallback must not leave non-interactive native views above Makepad
  controls.
- Logs must distinguish panel backend fallback from native-control fallback.

## Completion Rule

Native interactive controls are not complete until all gates above have working
prototype evidence on macOS and an SDK/runtime validation plan for UIKit.
