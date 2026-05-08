spec: task
name: "AI Chat Liquid Glass Step 59 aichat Scroll Edge Wiring"
tags: [makepad, liquid-glass, scroll-edge, aichat, phase-h]
---

## Intent

Wire the Makepad-rendered `GlassScrollEdge` widgets into aichat's chat list
overlay without adding native panels. This is a staged implementation: edges
are hidden for empty state and visible for non-empty chat content, but their
strength is not yet driven by precise scroll offset.

## Decisions

- Place scroll edge widgets inside `chat_shell`, above `ChatList`.
- Keep scroll edge widgets hidden while the empty state is visible.
- Do not export scroll edge widgets as native descriptors.
- Defer scroll-offset-driven intensity to a later step.

## Boundaries

### Allowed Changes

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`
- `examples/aichat/specs/APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`
- `examples/aichat/specs/aichat-liquid-glass-completion-audit.md`
- this step spec

### Constraints

- Do not create native panels.
- Do not change `ChatList` virtualization.
- Do not claim scroll-state-driven edge strength is complete.

## Acceptance Criteria

### Scenario: chat shell contains scroll edge widgets
Given aichat has a chat list overlay
When the aichat UI source is inspected
Then `scroll_top_edge` and `scroll_bottom_edge_host` are present near `chat_list`
Test: `rg "scroll_top_edge|scroll_bottom_edge_host|GlassScrollEdgeBottom" examples/aichat/src/main.rs`

### Scenario: empty state controls scroll edge visibility
Given scroll edges should not appear on the empty state
When `update_empty_state_visibility` is inspected
Then it sets scroll edge visibility to the inverse of `show_empty_state`
Test: `rg "show_scroll_edges|scroll_top_edge|scroll_bottom_edge_host" examples/aichat/src/main.rs`

### Scenario: scroll edges remain Makepad-rendered
Given scroll edge widgets must not add native panels
When the aichat UI source is inspected
Then the scroll edge instances do not set `native: true`
Test: `rg "scroll_top_edge|scroll_bottom_edge_host" examples/aichat/src/main.rs && ! rg "scroll_(top_edge|bottom_edge_host)[^\\{]*\\{[^}]*native: true" examples/aichat/src/main.rs`

### Scenario: staged limitation is documented
Given scroll offset is not wired yet
When policy and audit docs are inspected
Then they say aichat wiring exists but scroll-state-driven strength remains future work
Test: `rg "scroll-state-driven|scroll offset|aichat wiring" examples/aichat/specs/APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md examples/aichat/specs/aichat-liquid-glass-completion-audit.md`

### Scenario: aichat compiles
Given scroll edge widgets are wired into aichat DSL
When aichat is checked
Then it compiles
Test: `cargo check -p makepad-example-aichat --release`
