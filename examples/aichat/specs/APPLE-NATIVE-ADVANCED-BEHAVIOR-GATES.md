# Apple Native Advanced Behavior Gates

## Status

Implementation gates for Phase H and Phase I. This document does not claim
advanced native behavior is implemented.

## Scope

These gates cover the native Apple Liquid Glass behaviors that are not proven by
the current macOS `AppleNativeUnderlay` path:

- animated container spacing and morph transitions
- scroll edge glass
- fullscreen handling
- multi-display handling
- Stage Manager / split-view style window movement
- inactive-window native behavior
- popup and modal native glass surfaces

The current production path remains:

- one native container per window
- passthrough native panels below the Makepad Metal view
- Makepad-owned input and semantic controls
- shader fallback on unsupported platforms or reserved backends

## Gate Matrix

| Feature | Current evidence | Current status | Implementation gate |
|---|---|---|---|
| Animated spacing / morph transitions | `GlassContainer.spacing` maps to the descriptor model; Step 56 logs `native-container-spacing` and tests spacing changes invalidate native batches | Static mapping plus probe only; animated morph transitions remain unproven | Prove animated spacing can update native container spacing without frame drift, stale panels, or input regressions. |
| Scroll edge glass | `APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md` defines the route; `GlassScrollEdge` / `GlassScrollEdgeBottom` provide Makepad-rendered semantic hooks; Steps 60-62 wire edge visibility and strength to `PortalList` state | Makepad semantic scroll edges are wired; native scroll edge behavior remains future work | Define whether future native scroll edge glass is a native panel region or platform-specific control behavior. |
| Fullscreen | `WindowGeom.is_fullscreen` exists; Step 53 suppresses the current native underlay while fullscreen and restores it on exit; Step 65 adds `makepad-example-aichat-macos-native-clear-fullscreen-probe`; Step 134 records Studio build `[155]` with fullscreen enter, shader fallback, requested exit, native restore, and observed exit | Explicit fullscreen fallback validated; full native fullscreen glass remains out of scope | Keep the fallback/restore behavior stable; full native glass inside fullscreen is a later phase. |
| Multi-display | `WindowGeom.position`, `inner_size`, and `dpi_factor` exist; Step 54 logs `native-display-change` when backing-scale changes while native is active; Step 66 logs `native-display-frame-snapshot` / `native-panel-frame` from the cached native batch | Probe plus frame snapshot only; multi-display support remains unproven | Prove moving a native-glass window between displays recomputes frames in logical units and handles backing-scale changes. |
| Stage Manager / split view | Step 67 adds `makepad-example-aichat-macos-native-clear-geometry-probe` with `MAKEPAD_NATIVE_GLASS_GEOMETRY_SNAPSHOT=1` so same-screen position/size changes can log native frame snapshots | Probe only; Stage Manager and split-view behavior remain unproven | Smoke-test window resize/reposition sequences that resemble Stage Manager and record expected artifacts or supported behavior. |
| Inactive window behavior | Step 55 keeps shader inactive dimming at `0.70` and uses `NATIVE_INACTIVE_GLASS_MULTIPLIER` for a weaker native inactive app-side dim | Readability policy only; native inactive-window support remains unproven | Decide whether native glass should rely on system inactive behavior, app-side foreground tokens, or both; validate active/inactive transitions visually. |
| Popup/modal native glass | Makepad has popup/modal widgets and popup windows; no native glass transient-window policy exists | Not implemented | Define separate-window z-order, focus, dismissal, hit-test, and native surface ownership before creating AppKit `NSPanel` or UIKit `UIWindow` glass. |

## Required Evidence Before Marking Complete

### Animated spacing

- A test or Studio run shows changing spacing while panels stay aligned.
- Logs identify the container id and updated spacing.
- Resize after an animated spacing change does not leave stale panels.
- The current Step 56 spacing probe verifies batch invalidation and logs
  `native-container-spacing`.
- Step 63 adds the `makepad-example-aichat-macos-native-clear-spacing-probe`
  Studio runnable and `AICHAT_NATIVE_SPACING_PROBE=animate` app-side animation
  to exercise repeated native spacing updates.
- Step 129 records Studio release build `[140]` for the spacing animation
  probe. It ran frame `0 -> 120`, spacing `12 -> 36 -> 12`, and repeatedly
  logged `native-container-spacing` while the native batch remained Installed.
  This proves the runtime update path but not the final morphing visual quality.

### Scroll edge glass

- A long aichat conversation shows top and bottom edge treatment responding to
  scroll state.
- The treatment is visual-only and does not create per-row or per-message native
  panels.
- Studio widget dumps still show Makepad-owned scroll content and edge overlays.
- The current policy starts with Makepad semantic styling; native scroll edge
  behavior remains future work.
- `GlassScrollEdge` exists and is wired into aichat. Visibility follows
  `PortalList` state. Top edge opacity follows scroll offset; bottom edge
  opacity follows `PortalListRef::bottom_scroll_remaining()`.

### Fullscreen

- A release Studio run enters fullscreen and exits fullscreen on macOS.
- Native panel frames continue to match Makepad panel geometry.
- If fullscreen is disabled for a native backend, the disable path is explicit
  and logged instead of silently degrading.
- The current Step 53 fallback logs `fullscreen-native-fallback=shader` on
  enter and `fullscreen-native-restore=apple-native-underlay` on exit; this is
  not full native fullscreen support.
- Step 65 adds a fullscreen fallback probe runnable that requests fullscreen
  after native install, observes enter/exit geometry events, and exits again
  through the normal Makepad window path. If Studio or the OS does not emit a
  fullscreen-enter geometry event, the probe logs
  `native-fullscreen-probe=timeout phase=enter`.
- Step 128 records Studio release build `[139]` for that fullscreen probe. The
  app reached State 4 and requested fullscreen, but the probe timed out waiting
  for fullscreen-enter geometry. No fallback, restore, exit, or native panel
  frame evidence was produced, so fullscreen remains unproven.
- Step 134 fixes the fullscreen probe path. Studio release build `[155]`
  logged `native-fullscreen-op=enter old_fullscreen=false new_fullscreen=true`,
  `fullscreen-native-fallback=shader reason=fullscreen-enter`,
  `native-fullscreen-probe=observed-enter`,
  `native-fullscreen-probe=request-exit`,
  `fullscreen-native-restore=apple-native-underlay reason=fullscreen-exit`, and
  `native-fullscreen-probe=observed-exit`.

### Multi-display

- A validation run records old and new `dpi_factor`, window position, and panel
  frame coordinates after moving between displays.
- Native frames are computed from Makepad logical units after the backing-scale
  change.
- The result is documented as supported or as an explicit fallback.
- The current Step 54 probe records `native-display-change` for backing-scale
  changes, but it does not by itself prove panel alignment or complete
  multi-display support.
- Step 66 adds macOS backend frame snapshots on backing-scale changes:
  `native-display-frame-snapshot` summarizes dpi/position, and
  `native-panel-frame` logs each panel's Makepad logical rect plus the AppKit
  frame produced from it.

### Stage Manager / split view

- A release Studio run with
  `makepad-example-aichat-macos-native-clear-geometry-probe` records
  `native-display-frame-snapshot reason=geometry-change` while the window is
  resized or repositioned.
- The frame snapshots show native panels still using Makepad logical rects and
  AppKit frames that match the current container.
- The current Step 67 probe records evidence only; Stage Manager and split-view
  support remain unproven until a real runtime smoke run is reviewed.
- Step 131 records Studio release build `[142]` for the geometry probe. The app
  reached State 4, but System Events reported no scriptable
  `makepad-example-aichat` window, so no resize/reposition happened and no
  `native-display-frame-snapshot reason=geometry-change` evidence was produced.
- Step 132 makes the geometry probe self-driven by scheduling a next-frame
  resize/reposition request after `installed-native-glass-batch`, and macOS
  programmatic resize/reposition ops now emit `WindowGeomChange` with explicit
  old geometry. Studio builds `[146]` and `[147]` logged the request, but still
  produced no native frame snapshot evidence.
- Step 133 fixes the programmatic geometry event path by routing resize and
  reposition ops through the same internal `WindowGeomChange` handler instead
  of reentering `MacosApp::do_callback`. Studio release build `[150]` logged
  `native-geometry-op=resize changed=true`,
  `native-geometry-op=reposition changed=true`, and two
  `native-display-frame-snapshot reason=geometry-change` entries followed by
  four `native-panel-frame` entries each.

### Inactive window

- Active and inactive native-glass windows remain readable on bright and dark
  wallpapers.
- Makepad foreground tokens and native system behavior do not double-dim text.
- The inactive policy is shared by shader and Apple-native modes where possible.
- The current native inactive policy uses weaker app-side dimming than shader
  mode, but it does not prove complete native inactive-window support.
- Step 64 adds the `makepad-example-aichat-macos-native-clear-inactive-probe`
  Studio runnable and `AICHAT_NATIVE_INACTIVE_PROBE=1` logs for native active
  state, style, and multiplier.
- Step 130 records Studio release build `[141]` for the inactive probe. It
  logged `active=true style=clear multiplier=1.000`, but activating Finder did
  not produce `active=false` evidence in the Studio log stream. Inactive-window
  behavior remains unproven.
- Step 135 adds app-level activation events:
  `applicationDidBecomeActive:` maps to `WindowGotFocus`, and
  `applicationDidResignActive:` maps to `WindowLostFocus`. Studio release build
  `[157]` still could not produce `active=false` evidence because the current
  automation environment did not allow Finder to become the frontmost process.

### Popup/modal

- Transient windows have a separate native glass ownership policy from the main
  window.
- Makepad retains dismissal and focus semantics.
- Native transient surfaces do not intercept input outside their explicit
  interactive controls.

## Non-Completion Signals

The following are not enough to claim Phase H or Phase I support:

- `state=4` for the main window underlay.
- Static native panels aligned during normal resize.
- Makepad-only inactive tint tests.
- Existing popup/modal widgets working without native glass surfaces.
- A shader-backdrop proof showing interior blur/refraction.

## Recommended Implementation Order

1. Fullscreen explicit support or explicit native fallback.
2. Multi-display frame and backing-scale validation.
3. Inactive-window foreground/native behavior policy.
4. Animated spacing/morph transitions.
5. Scroll edge glass semantics.
6. Popup/modal separate-window native glass design.

This order keeps main-window correctness ahead of transient-window and animation
features.
