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

This keeps the landed AppleNativeUnderlay path safe: AppKit native glass panels
do not become input owners, and Makepad continues to handle text, scroll,
clicks, command menus, drag, generated Splash UI, and Studio inspection.

## Current Production Policy

- `GlassButton`, `GlassIconButton`, `GlassPrimaryButton`, `GlassNavButton`, and
  `GlassUtilityButton` are Makepad-rendered controls.
- Native glass panels are visual surfaces only.
- `NativeGlassHitTest::Interactive` must stay rejected until the gates below are
  implemented and validated.
- Native button research lives in `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; it is
  not implementation evidence.

## Future Gates

### Hit-test gate

- Decide whether AppKit/UIKit directly own native-control hit testing or
  whether Makepad routes events and forwards them to native controls.
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
