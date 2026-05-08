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
| Animated spacing / morph transitions | `GlassContainer.spacing` maps to the descriptor model; static spacing is part of Phase C | Static descriptor field only | Prove animated spacing can update native container spacing without frame drift, stale panels, or input regressions. |
| Scroll edge glass | Scroll widgets remain Makepad-rendered; no native scroll-edge descriptor exists | Not implemented | Define whether scroll edge glass is a semantic Makepad style, a native panel region, or a platform-specific control behavior. |
| Fullscreen | `WindowGeom.is_fullscreen` exists; release notes list fullscreen as a limitation | Not supported as native glass guarantee | Studio/manual run must enter and exit fullscreen with native panels aligned, no stale AppKit views, no crash, and explicit fallback logs if disabled. |
| Multi-display | `WindowGeom.position`, `inner_size`, and `dpi_factor` exist; no native glass display migration validation exists | Not implemented | Prove moving a native-glass window between displays recomputes frames in logical units and handles backing-scale changes. |
| Stage Manager / split view | No local runtime evidence beyond the known limitation list | Not implemented | Smoke-test window resize/reposition sequences that resemble Stage Manager and record expected artifacts or supported behavior. |
| Inactive window behavior | aichat applies a Makepad-only inactive multiplier on focus changes | Partial Makepad styling only | Decide whether native glass should rely on system inactive behavior, app-side foreground tokens, or both; validate active/inactive transitions visually. |
| Popup/modal native glass | Makepad has popup/modal widgets and popup windows; no native glass transient-window policy exists | Not implemented | Define separate-window z-order, focus, dismissal, hit-test, and native surface ownership before creating AppKit `NSPanel` or UIKit `UIWindow` glass. |

## Required Evidence Before Marking Complete

### Animated spacing

- A test or Studio run shows changing spacing while panels stay aligned.
- Logs identify the container id and updated spacing.
- Resize after an animated spacing change does not leave stale panels.

### Fullscreen

- A release Studio run enters fullscreen and exits fullscreen on macOS.
- Native panel frames continue to match Makepad panel geometry.
- If fullscreen is disabled for a native backend, the disable path is explicit
  and logged instead of silently degrading.

### Multi-display

- A validation run records old and new `dpi_factor`, window position, and panel
  frame coordinates after moving between displays.
- Native frames are computed from Makepad logical units after the backing-scale
  change.
- The result is documented as supported or as an explicit fallback.

### Inactive window

- Active and inactive native-glass windows remain readable on bright and dark
  wallpapers.
- Makepad foreground tokens and native system behavior do not double-dim text.
- The inactive policy is shared by shader and Apple-native modes where possible.

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
