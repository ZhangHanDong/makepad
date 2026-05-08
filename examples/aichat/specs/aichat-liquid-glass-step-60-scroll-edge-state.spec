spec: task
name: "AI Chat Liquid Glass Step 60 Scroll Edge State"
tags: [makepad, liquid-glass, scroll-edge, aichat, portal-list]
---

## Intent

Drive aichat scroll edge visibility from `PortalList` scroll state. This keeps
the implementation Makepad-rendered and avoids native panels while making the
edge overlays respond to whether content exists above or below the viewport.

## Decisions

- Hide both scroll edges for empty state.
- Show the top edge only after the list has scrolled away from the top.
- Show the bottom edge only while more items exist below the viewport.
- Refresh edge visibility on `PortalListAction::Scroll`.

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
- Do not alter chat item virtualization.

## Acceptance Criteria

### Scenario: empty state hides scroll edges
Given the chat empty state is visible
When scroll edge visibility is computed with stale scrolled-list state
Then both edges are hidden
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_state_hides_edges_for_empty_state --release`

### Scenario: top edge follows scroll-away-from-top state
Given the chat list is not empty
When the first visible item or first scroll offset indicates content above
Then the top edge is visible
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_state_shows_top_after_scroll --release`

### Scenario: bottom edge follows further-items state
Given the chat list is not empty
When further items exist below the viewport
Then the bottom edge is visible
Test: `cargo test -p makepad-example-aichat aichat_scroll_edge_state_shows_bottom_for_further_items --release`

### Scenario: PortalList scroll actions refresh edges
Given scroll edges are driven by list state
When app actions are handled
Then `list.scrolled(actions)` triggers scroll edge visibility refresh
Test: `rg "list\\.scrolled\\(actions\\)|update_scroll_edge_visibility" examples/aichat/src/main.rs`

### Scenario: scroll edges do not create native panels
Given scroll edge treatment is semantic and Makepad-rendered
When the implementation is inspected
Then `GlassScrollEdge` is used without adding native panel descriptors
Test: `rg "GlassScrollEdge|NativeGlassPanelDescriptor" examples/aichat/src/main.rs widgets/src/glass_panel.rs`

### Scenario: PortalList internals stay unchanged
Given scroll edge state uses the public PortalList reference API
When the worktree diff is inspected
Then `widgets/src/portal_list.rs` and PortalList internals are unchanged
Test: `test -z "$(git diff --name-only -- widgets/src/portal_list.rs widgets/src/portal_list_ref.rs)"`

### Scenario: aichat compiles
Given scroll edge visibility is wired to PortalList state
When aichat is checked
Then it compiles
Test: `cargo check -p makepad-example-aichat --release`
