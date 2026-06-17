# Apple Native Liquid Glass Full Integration Roadmap

Date: 2026-05-27

This roadmap tracks the remaining work to turn the current Apple native Liquid
Glass vertical slice into a complete Makepad native API integration. It covers
the Apple native path only. Whole-panel shader refraction, water-like warping,
and cross-platform backdrop sampling remain part of the separate
ShaderBackdrop route.

## Current Status

The macOS native path is proven end to end in a standalone example:

- `makepad-example-native-liquid-glass` runs outside aichat layout constraints.
- `GlassContainer { native: true }` installs one native macOS container with
  three native panels.
- `NSGlassEffectView` style, tint, rounded-rect radius, capsule shape, panel
  z-order, and descriptor updates are active.
- `GlassContainer.spacing` maps to native container spacing.
- `Button.native_control` can install an AppKit `NativeGlassButton`.
- Native button action delivery reaches Makepad actions.
- `Native Press` drives a multi-second descriptor pulse that updates spacing,
  radius, and tint over time.
- The standalone validation bench now includes dedicated `Geometry`,
  `Container`, `Debug`, `Readability`, and Makepad demo-overlay toggle paths.
- `GoldGlintEdge` is documented and toggled as a Makepad shader overlay, not
  Apple native API evidence.

The current proof is a vertical slice, not a complete product API. It proves
the route is viable and gives us a stable visual/debug target.

## Completion Definition

The Apple native path is complete when Makepad supports:

- Stable public widget properties for native panels, containers, and controls.
- macOS native panel/container/control installation with observable per-item
  results and predictable fallback.
- iOS/iPadOS native panel/container backend once an iOS 26 SDK/runtime is
  available.
- App-level integration in aichat without opaque Makepad surfaces covering the
  native glass.
- Documented limitations for behavior Apple native APIs do not expose.

## Track 1: Surface And Panel API

Already working:

- `NativeGlassStyle::Regular`
- `NativeGlassStyle::Clear`
- `NativeGlassShape::RoundedRect`
- `NativeGlassShape::Capsule`
- tint conversion through sRGB colors
- panel z-order inside the single native container
- descriptor update path from widget tree to platform backend
- focused descriptor equivalence tests for style, tint, radius, hit-test,
  z-order, visibility, and subpixel tolerance

Remaining work:

- Expose per-panel install result to logs or a queryable diagnostics structure.
- Tighten panel diffing so style/tint/radius updates do not reinstall more than
  necessary.
- Finalize defaults for `GlassPanel.native`, style, shape, radius, tint, and
  hit-test policy.
- Validate fallback behavior when a selector is missing at runtime.
- Keep `NativeGlassHitTest::Interactive` rejected for panels until a later
  phase explicitly changes the hit-test model.

Acceptance:

- A standalone run logs container and per-panel state clearly.
- Updating style, tint, radius, and z-order changes the native panel without
  breaking the Metal overlay.
- Existing non-native `GlassPanel` usage remains unchanged by default.

## Track 2: Container And Morph Behavior

Already working:

- one `GlassContainer` per window
- `GlassContainer.spacing` mapped to native macOS container spacing
- runtime descriptor pulse proves dynamic spacing updates can be delivered
- a dedicated standalone `Container` mode with Near / Threshold / Far spacing
  presets
- a standalone morph mode with near, far, and overlapping panel states

Remaining work:

- Validate adjacent panel merge behavior at multiple distances.
- Validate overlapping panels with different `Regular` / `Clear` styles.
- Validate morph behavior during window resize.
- Document the visual limits of Apple native morphing: strongest changes are
  expected around edges, corners, overlaps, and merge zones.
- Keep multi-container support out of the first complete macOS cut unless a real
  UI requires it.

Acceptance:

- Standalone morph mode demonstrates near, far, and overlapping panel states.
- Logs show stable native install/update behavior during morph animation.
- The roadmap can state which morph behaviors are Apple-provided and which are
  not available through native APIs.

## Track 3: Native Controls

Already working:

- `Button.native_control`
- `Button.native_control_role`
- `NativeGlassControlBatch`
- AppKit native button installation
- AppKit target/action bridge back into Makepad `ButtonAction::Clicked`
- standalone `Native Press` visual and action proof
- standalone native button role matrix for `Default`, `Primary`, `Utility`,
  `Icon`, and `Nav`

Remaining work:

- Verify disabled, hidden, focus, hover, press, and keyboard activation states.
- Decide whether role-specific appearance should be fully native or mixed with
  Makepad-rendered fallback.
- Add icon-only native control support or explicitly document it as unsupported.
- Reduce diagnostic/probe-only logs for normal app runs.
- Validate control frame updates during real backing-scale changes.
- Decide if `GlassButton` should opt into native controls directly, or if native
  control is a separate explicit property.

Acceptance:

- A standalone control matrix shows each role and its native behavior.
- Native control activation is validated through real click and probe paths.
- Makepad-rendered controls remain the default unless native controls are
  explicitly requested.

## Track 4: macOS Runtime Robustness

Already working:

- Normal install/state/result logs remain enabled for acceptance runs.
- Verbose native panel/container diagnostic logs are gated by
  `MAKEPAD_NATIVE_GLASS_DIAGNOSTIC_LOGS`.
- The standalone bench has a `Resize Probe` path that toggles the window between
  validation sizes and logs panel/control fit summaries after
  `WindowGeomChange`.
- Controls mode resize validation now requires the full five-control matrix to
  remain inside the window and logs `controls=5 controls_fit=true`.
- Resize validation summaries include `dpi=... scale_changed=...` so
  backing-scale transitions are auditable from runtime logs.

Remaining work:

- Broader manual resize validation across small and large windows.
- Fullscreen fallback policy.
- Stage Manager behavior.
- Real multi-display backing-scale changes.
- Active/inactive window material changes.
- Reduce Transparency behavior.
- Light and dark wallpaper readability.
- Performance budget for visible native panels and controls.

Acceptance:

- Runtime edge cases have explicit logs and documented behavior.
- Unsupported states fall back predictably instead of partially installing
  broken native views.
- Manual validation checklist covers bright wallpaper, dark wallpaper, resize,
  and inactive window behavior.

## Track 5: iOS And iPadOS UIKit Backend

Current blocker:

- Local SDK evidence previously showed iPhoneOS 18.5, which does not expose the
  typed iOS 26 `UIGlassEffect` / `UIGlassContainerEffect` symbols.

Remaining work after iOS 26 SDK/runtime is available:

- Validate `UIGlassEffect`.
- Validate `UIGlassContainerEffect`.
- Implement UIKit native panel/container installer.
- Map `GlassContainer.spacing` to UIKit container spacing.
- Validate UIKit clear/regular raw style mapping.
- Implement or explicitly defer native UIKit button glass configuration.
- Add an iOS standalone native Liquid Glass example.

Acceptance:

- iOS example shows multiple native panels through UIKit.
- iOS logs mirror macOS container/panel install result schema.
- iOS fallback is explicit when the runtime or SDK does not expose required API.

## Track 6: Popup And Modal Glass

Remaining work:

- Treat popup/modal glass as a separate platform-window phase.
- Define NSPanel/UIWindow ownership.
- Define focus, dismiss, hit-test, and z-order behavior.
- Validate that popup glass does not conflict with the main window native
  container.

Acceptance:

- One popup/modal proof runs independently from the main window panel path.
- Main-window glass and popup/modal glass have separate diagnostics.

## Track 7: aichat Integration

Already partially working:

- aichat has native backend switches and native panel wiring.
- standalone example avoids aichat opaque layout issues while proving native
  behavior.

Remaining work:

- Reconnect the stable native API to aichat shell surfaces.
- Add opaque guard for generated Splash / AI UI roots.
- Decide which aichat controls become native controls and which remain
  Makepad-rendered.
- Keep text readability surfaces when native glass alone is not readable.
- Ensure no opaque Makepad surface covers the native underlay.
- Keep aichat run items separate from standalone native validation run items.

Acceptance:

- aichat native mode shows real native glass without relying on diagnostic
  stripes or artificial backgrounds.
- aichat UI remains readable on bright and dark wallpapers.
- Native controls are opt-in and do not break Makepad input routing.

## Recommended Implementation Order

1. Stabilize the standalone native example as the visual acceptance target.
   Status: complete in `makepad-example-native-liquid-glass`.
2. Add a dedicated container morph mode.
   Status: complete in the standalone bench; visual merge/resize validation
   remains part of runtime robustness.
3. Add a native button role matrix.
   Status: complete in the standalone bench; deeper focus/keyboard/hover state
   validation remains.
4. Extract reusable descriptor pulse/control helpers into Makepad APIs or
   documented widget patterns.
5. Tighten macOS runtime robustness: resize, active/inactive, display changes,
   and fallback behavior.
6. Reconnect the stable subset to aichat.
7. Start iOS/iPadOS backend work only after an iOS 26 SDK/runtime is available.
8. Add popup/modal glass as a separate phase.

## Non-Goals For This Roadmap

- Do not mix ShaderBackdrop refraction into the Apple native backend.
- Do not make native controls the default for all buttons.
- Do not support arbitrary vector paths as native glass panel shapes.
- Do not support per-row native panels in lists.
- Do not add multi-container support until a concrete UI needs it.
