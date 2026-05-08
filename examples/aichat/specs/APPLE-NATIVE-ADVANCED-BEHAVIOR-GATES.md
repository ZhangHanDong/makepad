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
| Scroll edge glass | `APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md` defines the route; `GlassScrollEdge` / `GlassScrollEdgeBottom` provide Makepad-rendered semantic hooks | Widget hook only; not wired to scroll state | Define whether scroll edge glass is a semantic Makepad style, a native panel region, or a platform-specific control behavior. |
| Fullscreen | `WindowGeom.is_fullscreen` exists; Step 53 suppresses the current native underlay while fullscreen and restores it on exit | Explicit fullscreen fallback, not full native fullscreen support | Studio/manual run must enter and exit fullscreen with native panels aligned, no stale AppKit views, no crash, and explicit fallback logs if disabled. |
| Multi-display | `WindowGeom.position`, `inner_size`, and `dpi_factor` exist; Step 54 logs `native-display-change` when backing-scale changes while native is active | Probe only; multi-display support remains unproven | Prove moving a native-glass window between displays recomputes frames in logical units and handles backing-scale changes. |
| Stage Manager / split view | No local runtime evidence beyond the known limitation list | Not implemented | Smoke-test window resize/reposition sequences that resemble Stage Manager and record expected artifacts or supported behavior. |
| Inactive window behavior | Step 55 keeps shader inactive dimming at `0.70` and uses `NATIVE_INACTIVE_GLASS_MULTIPLIER` for a weaker native inactive app-side dim | Readability policy only; native inactive-window support remains unproven | Decide whether native glass should rely on system inactive behavior, app-side foreground tokens, or both; validate active/inactive transitions visually. |
| Popup/modal native glass | Makepad has popup/modal widgets and popup windows; no native glass transient-window policy exists | Not implemented | Define separate-window z-order, focus, dismissal, hit-test, and native surface ownership before creating AppKit `NSPanel` or UIKit `UIWindow` glass. |

## Required Evidence Before Marking Complete

### Animated spacing

- A test or Studio run shows changing spacing while panels stay aligned.
- Logs identify the container id and updated spacing.
- Resize after an animated spacing change does not leave stale panels.
- The current Step 56 spacing probe verifies batch invalidation and logs
  `native-container-spacing`, but it does not implement animated morph
  transitions.

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

### Multi-display

- A validation run records old and new `dpi_factor`, window position, and panel
  frame coordinates after moving between displays.
- Native frames are computed from Makepad logical units after the backing-scale
  change.
- The result is documented as supported or as an explicit fallback.
- The current Step 54 probe records `native-display-change` for backing-scale
  changes, but it does not by itself prove panel alignment or complete
  multi-display support.

### Inactive window

- Active and inactive native-glass windows remain readable on bright and dark
  wallpapers.
- Makepad foreground tokens and native system behavior do not double-dim text.
- The inactive policy is shared by shader and Apple-native modes where possible.
- The current native inactive policy uses weaker app-side dimming than shader
  mode, but it does not prove complete native inactive-window support.

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
