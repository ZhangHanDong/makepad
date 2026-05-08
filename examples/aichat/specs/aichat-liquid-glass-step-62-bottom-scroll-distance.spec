spec: task
name: "AI Chat Liquid Glass Step 62 Bottom Scroll Distance"
tags: [makepad, liquid-glass, scroll-edge, aichat, portal-list]
---

## Intent

Expose bottom scroll distance from `PortalList` so aichat can ramp bottom scroll
edge opacity continuously instead of using only a binary "more items below"
state.

## Decisions

- `PortalList` caches `bottom_scroll_remaining` during draw finalization.
- `PortalListRef::bottom_scroll_remaining()` exposes the cached value to app
  code.
- `0.0` means the list bottom is visible or content does not fill the viewport.
- `f64::INFINITY` means the last item was not drawn, so the bottom edge should
  be treated as fully active.
- aichat bottom edge alpha ramps from `bottom_scroll_remaining`.

## Boundaries

### Allowed Changes

- `widgets/src/portal_list.rs`
- `examples/aichat/src/main.rs`
- `examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not create native panels.
- Do not change PortalList item virtualization.
- Do not change PortalList action names or event routing.

## Acceptance Criteria

### Scenario: bottom alpha ramps from remaining distance
Given the bottom edge is visible
When bottom remaining distance is below the fade distance
Then bottom alpha is greater than zero and below the cap
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_alpha_ramps_bottom_from_remaining_distance --release`

### Scenario: distant bottom content caps edge alpha
Given content is definitely below the viewport
When bottom remaining distance reaches the fade distance
Then bottom edge alpha reaches the cap
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_alpha_caps_when_content_is_above_or_below --release`

### Scenario: PortalList exposes bottom distance
Given app code needs bottom scroll distance
When PortalList public reference APIs are inspected
Then `PortalListRef::bottom_scroll_remaining()` is available
Test: `rg "bottom_scroll_remaining" widgets/src/portal_list.rs examples/aichat/src/main.rs`

### Scenario: native panel budget is unchanged
Given scroll edge strength remains Makepad-rendered
When native glass code is inspected
Then no native panel descriptor is added for scroll edge strength
Test: `test -z "$(git diff --name-only -- widgets/src/glass_panel.rs)"`

### Scenario: aichat and widgets compile
Given PortalList exposes bottom distance and aichat consumes it
When the affected packages are checked
Then they compile
Test: `cargo check -p makepad-example-aichat --release`
