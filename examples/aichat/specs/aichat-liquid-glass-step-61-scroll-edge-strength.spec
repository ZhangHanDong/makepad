spec: task
name: "AI Chat Liquid Glass Step 61 Scroll Edge Strength"
tags: [makepad, liquid-glass, scroll-edge, aichat, portal-list]
---

## Intent

Make aichat scroll edge treatment less binary by deriving edge alpha from
`PortalList` state where that state has enough detail. The implementation stays
Makepad-rendered and does not add native panels.

## Decisions

- Top edge alpha ramps from `PortalList::scroll_position()` while the first item
  is still visible.
- Top edge alpha caps once `first_id > 0`, because content is definitely above
  the viewport.
- Bottom edge alpha remains capped/binary in this step because `PortalList`
  exposes `further_items_bellow_exist()` but not bottom distance.
- Edge color uses the same sRGB color as `GlassScrollEdge` with dynamic alpha.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not create native panels.
- Do not change `PortalList` internals.
- Do not claim bottom-edge continuous strength until bottom distance exists.

## Acceptance Criteria

### Scenario: hidden edges have zero alpha
Given edge visibility is false
When alpha is computed
Then top and bottom alpha are zero
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_alpha_is_zero_when_hidden --release`

### Scenario: top edge ramps from scroll offset
Given the top edge is visible and the first item is still visible
When the list has a partial negative scroll offset
Then top edge alpha is greater than zero and below the cap
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_alpha_ramps_top_from_scroll_offset --release`

### Scenario: definite content above or below caps edge alpha
Given content is definitely above or below the viewport
When alpha is computed
Then the corresponding edge alpha reaches the cap
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_alpha_caps_when_content_is_above_or_below --release`

### Scenario: dynamic alpha is applied to edge widgets
Given scroll edge alpha is computed from list state
When the app refreshes scroll edge state
Then both edge widgets receive dynamic `draw_bg.color` alpha
Test: `rg "chat_scroll_edge_alpha|scroll_bottom_edge|draw_bg \\+:" examples/aichat/src/main.rs`

### Scenario: edge strength does not create native panels
Given scroll edge strength is a Makepad-rendered treatment
When the implementation is inspected
Then no native panel descriptor is created for scroll edge strength
Test: `test -z "$(git diff --name-only -- widgets/src/glass_panel.rs)"`

### Scenario: PortalList internals stay unchanged
Given scroll edge strength uses public PortalList reference state
When the worktree diff is inspected
Then `widgets/src/portal_list.rs` and PortalList internals are unchanged
Test: `test -z "$(git diff --name-only -- widgets/src/portal_list.rs widgets/src/portal_list_ref.rs)"`

### Scenario: aichat compiles
Given scroll edge strength is wired to edge widgets
When aichat is checked
Then it compiles
Test: `cargo check -p makepad-example-aichat --release`
