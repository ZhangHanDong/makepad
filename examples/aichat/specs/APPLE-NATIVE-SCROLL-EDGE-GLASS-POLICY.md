# Apple Native Scroll Edge Glass Policy

## Status

Policy for Phase H scroll edge glass. This document does not implement scroll
edge glass.

## Current Evidence

- aichat scroll surfaces are Makepad-rendered.
- Native glass v4.1 supports one container per window and a small number of
  passthrough panels.
- The native panel budget is capped at 12 visible panels per window.
- Per-row or per-scroll-content native panels are explicitly out of scope.

## Decision

In the next implementation phase, scroll edge glass starts as a Makepad semantic
style, not as additional native panels.

The initial semantic model should be:

- A scroll view can expose top and bottom edge states.
- A glass-aware edge overlay can draw Makepad-rendered fade, separator, or
  subtle highlight treatment.
- Native Apple glass panels remain the large structural surfaces: shell,
  sidebar, main area, and composer.
- Scroll edge treatment must not create native panels for every row, message,
  or list item.

## Rationale

Apple scroll edge behavior is tied to platform navigation bars, toolbars, and
scroll container presentation. Makepad's current aichat layout keeps input,
Markdown, generated Splash UI, and list virtualization inside the Metal layer.
Creating native panels for scroll edges before defining scroll ownership would
increase z-order and hit-test risk without proving full native Liquid Glass.

Keeping scroll edge glass semantic first preserves:

- Makepad-owned scrolling and input.
- The native panel budget.
- One-container-per-window v4.1 constraints.
- Cross-platform shader fallback behavior.

## Future Native Gate

Scroll edge glass can move toward native behavior only after these questions are
answered:

- Which widget owns scroll edge state: `ScrollYView`, `PortalList`, a wrapper,
  or the app?
- Are scroll edges visual-only overlays, native toolbar/nav-bar behaviors, or
  extra native glass regions?
- How are edge overlays clipped when the scroll content has rounded corners?
- How do edge states interact with fullscreen, popup/modal surfaces, and
  multi-display moves?
- How is the behavior represented in Studio widget dumps and screenshots?

## Non-Completion Signals

The following are not enough to claim scroll edge glass support:

- Existing `GlassSeparator` usage.
- Static native panels behind a scroll view.
- ShaderBackdrop interior refraction.
- A scroll view that merely sits inside a native glass panel.

## Recommended First Implementation

1. Add a Makepad-rendered `GlassScrollEdge` or equivalent style hook.
2. Drive it from scroll state exposed by the owning scroll widget or wrapper.
3. Keep it visual-only and input-passthrough.
4. Validate with aichat long conversation scrolling.
5. Only after that, decide whether platform-native scroll edge integration is
   needed.

## Current Implementation Step

`GlassScrollEdge` and `GlassScrollEdgeBottom` now exist as Makepad-rendered
semantic widgets. They are not native panels.

Step 59 adds aichat wiring in the chat list overlay. Step 60 drives edge
visibility from `PortalList` state: the top edge appears after scrolling away
from the top, and the bottom edge appears while further items exist below the
viewport. Edge opacity/strength is still binary rather than continuously
scroll-offset-driven.
