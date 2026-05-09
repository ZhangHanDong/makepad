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

Step 106 adds descriptor content needed by a native button installer:

- `NativeGlassControlDescriptor.label: String`
- `Button.native_control` copies `Button.text` into that label

Icon/image transport remains out of scope, so icon-only native controls can use
an empty label until a later descriptor slice adds image data.

Step 107 makes macOS consume control batches for diagnostics:

- `CxOsOp::SetNativeGlassControlBatch` forwards to
  `MacosWindow::update_native_glass_control_batch`
- valid batches log `state=Unsupported reason=installer-not-implemented`
- invalid batches log stable rejection reasons

This is intentionally not AppKit control installation yet; it proves the op
reaches the macOS backend and remains honest about missing native controls.

Step 108 adds the first macOS AppKit installer skeleton:

- visible native button descriptors create `NSButton` mirrors above the Metal
  view
- `NativeGlassControlTarget` posts `NativeGlassControlActivatedEvent`
- descriptor labels become `NSButton` titles

This proves the native-control hit-test/action ownership path on macOS, but it
does not prove macOS 26 glass-specific `NSButton` styling. aichat remains
opt-in; no app buttons are enabled as native controls by default.

Step 109 adds an aichat runtime probe gate:

- `AICHAT_NATIVE_CONTROL_PROBE=buttons|1|true|on`
- startup marks `clear_button` and `send_button` as `native_control: true`

This gives Studio a runnable path for macOS native-control validation while
keeping default aichat behavior unchanged.

Step 110 exposes that probe through Studio:

- `makepad-example-aichat-macos-native-clear-control-probe`
- `AICHAT_NATIVE_CONTROL_PROBE=buttons`

This keeps the required RunItem validation path available without using bridge
Cargo requests or changing the default native-clear runnable.

Step 111 records the first runtime result from that runnable:

- build `[111]`
- `backend=apple-native-controls state=Installed
  reason=installed-appkit-buttons controls_total=2 controls_visible=2`

This proves descriptor delivery and AppKit button installation, but it does not
yet prove real mouse-driven target/action delivery because Studio click
injection bypasses AppKit overlay hit testing.

Step 112 adds diagnostics for that missing verdict:

- `event=target-action` when AppKit calls `nativeGlassControlAction:`
- `event=button-action` when Makepad `Button` converts the native activation
  into `ButtonAction::Clicked`

The real-click validation must produce both lines for the clicked control.

Step 113 moves macOS native controls to the top of the AppKit container view
with `relativeTo:nil`. Visual glass panels stay below Metal; only explicit
native controls get this topmost insertion path.

Step 114 adds aichat-side probe click diagnostics:

- `native-control-probe=makepad-click id=clear_button`
- `native-control-probe=makepad-click id=send_button`

These lines prove that a system click reached the Makepad button path. They do
not prove AppKit native control delivery unless paired with `event=target-action`
and `event=button-action`.

Step 115 tried system-level `cliclick` validation against build `[114]`. Startup
still installed two AppKit controls, but the click attempts produced no
`target-action`, `button-action`, or `makepad-click` logs. Treat that result as
failed/inconclusive validation, not as completion.

Step 116 replaces raw `NSButton` allocation with a `NativeGlassButton` subclass
for diagnostics:

- `acceptsFirstMouse:` returns `YES` and logs `event=accepts-first-mouse`
- `hitTest:` logs `event=button-hit-test`
- `mouseDown:` logs `event=button-mouse-down`

The next click attempt should reveal whether AppKit hit testing reaches the
native button at all.

Step 117 adds probe-gated Metal view diagnostics:

- `event=metal-view-hit-test`
- `event=metal-view-mouse-down`

If these lines appear without native button logs, the click is reaching the
Makepad view instead of the AppKit button. If neither class of log appears, the
local system-click automation is not reaching the Studio-launched app window.

Step 118 records that build `[116]` produced no post-click diagnostics after
system-level and Studio click attempts, confirmed with `QueryLogs` since index
`8154`. This keeps native-control validation open.

Step 119 adds AppKit container hierarchy logs after native control installation:

- `hierarchy subviews=N`
- `hierarchy index=... role=... class=... frame=(...) hidden=...`

This verifies native button placement and sibling order, but it is not a
substitute for real AppKit target/action delivery.

Step 119 found the native frame for `clear_button` did not match the Studio
`WidgetQuery` rect. A `clipped_rect` attempt rejected the control batch as
`empty-visible-control-rect`; a view-origin transform attempt produced invalid
offscreen frames. Neither geometry fix is retained.

Step 121 adds `native-control-frame` logs with `label` so runtime evidence can
confirm the installed AppKit controls are the intended Makepad buttons. The
same run also clarifies that Studio `WidgetQuery` / `Click` coordinates live in
Studio remote input space and must not be treated as AppKit subview frames.
Studio framebuffer screenshots do not include AppKit sibling views, so they are
not sufficient native-control visual proof.

This keeps the landed AppleNativeUnderlay path safe: AppKit native glass panels
do not become input owners, and Makepad continues to handle text, scroll,
clicks, command menus, drag, generated Splash UI, and Studio inspection.

## Current Production Policy

- `GlassButton`, `GlassIconButton`, `GlassPrimaryButton`, `GlassNavButton`, and
  `GlassUtilityButton` are Makepad-rendered controls.
- Native glass panels are visual surfaces only.
- `NativeGlassHitTest::Interactive` must stay rejected until the gates below are
  implemented and validated.
- `NativeGlassControlBatch` is the native-control input model; it is separate
  from `NativeGlassBatch`.
- `SetNativeGlassControlBatch` may be queued by future collectors, but current
  non-macOS backends treat it as a no-op.
- `GlassContainer` can carry native-control descriptors from explicit
  `Button.native_control` opt-ins, but current `GlassButton` variants remain
  Makepad-rendered only.
- `Button.native_control` can export descriptors for native button mirrors, but
  it is disabled by default and only the macOS `NSButton` installer slice
  exists.
- `NativeGlassControlActivatedEvent` is the only app-facing native control
  activation bridge for the first button slice; widgets translate it back into
  their existing action APIs.
- `NativeGlassControlDescriptor.label` is the platform-neutral title payload for
  the first native button installer; icon transport is not part of this slice.
- macOS validates and logs native control batches but does not create AppKit
  controls unless explicit `Button.native_control` descriptors are present.
- macOS AppKit control installation currently mirrors only `NSButton` title,
  enabled/hidden state, rect, and target/action; glass-specific button styling
  remains unproven.
- aichat native controls are opt-in behind `AICHAT_NATIVE_CONTROL_PROBE`.
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
