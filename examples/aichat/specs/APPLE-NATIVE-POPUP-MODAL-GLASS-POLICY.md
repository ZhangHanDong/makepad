# Apple Native Popup and Modal Glass Policy

Date: 2026-05-09

## Scope

This policy covers Phase I native glass for transient surfaces:

- Makepad popup windows created through `CxOsOp::CreatePopupWindow`.
- Future modal surfaces that are promoted to separate platform windows.

It does not change the v4.1 main-window model: one native container below one
Metal layer, with Makepad-owned input for ordinary glass panels.

## Current Platform Shape

Makepad popup windows already use separate platform windows:

- `platform/src/window.rs::WindowHandle::new_popup` creates a popup `WindowId`
  and emits `CxOsOp::CreatePopupWindow`.
- macOS handles that op with `MetalWindow::new_popup`.
- `MacosWindow::init_popup` creates a borderless `NSPanel` at popup-menu level,
  sets `container_view` as the content root, adds the Metal view as a subview,
  and relies on `PopupDismissed` events for dismissal.

This means popup/modal native glass must be owned by the transient window, not
by the main window's native glass container.

## Decisions

1. Transient windows get independent native glass ownership.
   A popup or modal must not reuse the main window's `NSGlassEffectContainerView`
   or UIKit visual-effect container.

2. The main window and transient windows do not share z-order.
   The platform window manager orders the transient window above its parent.
   Within a transient window, native glass remains below that window's Metal
   view unless a future explicit native control descriptor opts into hit testing.

3. Popup dismissal stays Makepad-owned.
   Native glass panels in transient windows are passthrough by default. Outside
   click, Escape, focus loss, and parent-close behavior continue to flow through
   the existing `PopupDismissed` policy.

4. Modal focus stays Makepad-owned until a separate modal-window phase.
   A Makepad-rendered modal inside the main window does not create native
   transient glass. Only a future separate-platform-window modal can own an
   independent native glass container.

5. No native controls are installed by popup/modal glass v1.
   Native popup/modal surfaces start as passthrough panels. Interactive native
   buttons, text fields, or menu rows require the native-control policy and a
   separate hit-test bridge.

6. Descriptor limits are per platform window.
   Main-window panel limits do not aggregate with a popup window. Each transient
   window still follows the v4.1 single-container limit and the per-window panel
   budget.

## macOS Implementation Plan

Phase I should add a small transient-window native substrate path:

1. Add a popup/modal native-glass probe gate, for example
   `MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE=1`.
2. In `CxOsOp::CreatePopupWindow`, after `MetalWindow::new_popup` and
   `set_window_visuals`, install a native glass substrate only for the popup
   window when the probe is enabled.
3. Keep the popup Metal view above the native glass view.
4. Log the transient result with a distinct schema:

```text
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear
```

5. Do not export main-window `GlassPanel` descriptors into the popup window.
   Popup descriptors must come from the popup window's own widget tree.

## macOS Probe Evidence

Step 136 adds `makepad-example-aichat-macos-native-clear-transient-probe`.
The run item sets `MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE=1`; aichat opens a
real platform popup after the main native substrate reaches State 4.

Studio release builds `[161]` and `[166]` logged:

```text
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
[liquid-glass] transient-window-probe=draw popup_size=(240.0,160.0)
[liquid-glass] transient-window=popup-dismiss-probe request=dispatch-popup-dismissed
[liquid-glass] transient-window-probe=dismissed popup=WindowId(1, 0) reason=FocusLost
```

The same run kept the main native batch installed:

```text
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The first probe proves macOS transient-window native substrate installation.
Studio screenshot
`/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-161-kind-0-req-49-1778342316874.png`
captured the popup framebuffer with the probe marker content. It does not yet
prove popup widget-tree descriptor collection. Build `[166]` proves the
platform-to-app `PopupDismissed(FocusLost)` delivery path through a synthetic
platform dispatch; a physical outside-click or Escape dismissal remains a later
manual/runtime validation.

Step 137 changes the same transient probe to draw a real popup-local
`GlassContainer` widget tree. Studio release build `[168]` logged:

```text
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] native-container-spacing container=0000000000000047 spacing=10.000
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
[liquid-glass] container=0000000000000047 state=Installed reason=installed panels_installed=1 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

The main native batch remained installed with four panels in the same run.
This proves popup widget-tree descriptor export for the macOS probe path.

## UIKit Implementation Plan

UIKit popup/modal support must be separate from the current main `UIWindow`
installer:

1. A modal or popover that is represented by a separate `UIWindow` owns its own
   `UIVisualEffectView` host.
2. The main window's UIKit native glass batch must not be reused by transient
   windows.
3. Keyboard, safe-area, and scene activation behavior must be validated before
   enabling the UIKit path by default.

## Acceptance

Phase I is not complete until all of the following are true:

- A macOS Studio release run opens a popup window with transient native glass
  installed and logs the transient schema.
- Main-window native panels remain installed and aligned while the popup is
  open and after it is dismissed.
- Popup dismissal still emits the existing `PopupDismissed` semantics.
- The popup's native glass is passthrough unless explicit native controls are
  installed.
- UIKit either has equivalent runtime evidence or logs a stable unsupported
  reason.

## Non-Goals

- No nested native containers inside the main window for popups.
- No native per-row popup menu items in the first transient phase.
- No native modal text fields or buttons until native controls are complete.
- No claim that existing Makepad-rendered `Modal` widgets are native glass.
