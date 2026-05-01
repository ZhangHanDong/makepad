# aichat Liquid Glass Opaque Pixel Audit

Source task spec: [AICHAT-LIQUID-GLASS-IMPLEMENTATION-TASKS.md](AICHAT-LIQUID-GLASS-IMPLEMENTATION-TASKS.md)

## Status

Phase 0 initial audit. This document classifies known opaque or semi-opaque drawing surfaces that can hide a future native macOS glass substrate.

Native substrate implementation remains blocked until this audit and the Phase 1.5 window hierarchy proof are complete.

## Categories

| Category | Meaning |
|---|---|
| Accidental large opaque surface | A full-window or large region fill that can hide native glass without being intentional content. |
| Panel substrate overlay | A `GlassPanel` tint/decorative layer that should become preset-driven. |
| Local readability surface | A deliberately stronger surface that preserves text/code readability. |
| Dynamic generated UI | LLM/Splash-generated content that cannot be fully audited at build time. |

## Audit Table

| Surface | Location | Category | Current Alpha Behavior | Native-Mode Strategy |
|---|---|---|---|---|
| Window pass clear | `examples/aichat/src/main.rs`, `pass.clear_color: #00000000` | Accidental large opaque surface | Fully transparent. Correct precondition. | Keep transparent. Any native mode must preserve alpha 0. |
| Window transparency | `examples/aichat/src/main.rs`, `window.transparent: true` | Accidental large opaque surface | Requests transparent platform window. Correct precondition. | Keep enabled for native substrate. |
| Root body background | `examples/aichat/src/main.rs`, `body.draw_bg.color: #00000000` | Accidental large opaque surface | Fully transparent root. Correct precondition. | Keep transparent. |
| `app_shell` | `examples/aichat/src/main.rs`, `app_shell := GlassPanel` | Panel substrate overlay | Tint/decorative panel; previously driven only by `tint_alpha`. | Phase 1 makes tint, border, highlight, noise, and halo preset-driven. |
| `sidebar` | `examples/aichat/src/main.rs`, `sidebar := GlassPanel` | Panel substrate overlay | Tint/decorative panel; stronger than outer shell. | Phase 1 makes preset-driven; native mode lowers tint and decoration strength. |
| `main_area` | `examples/aichat/src/main.rs`, `main_area := GlassPanel` | Panel substrate overlay | Tint/decorative main content substrate. | Phase 1 makes preset-driven; native mode lowers tint and decoration strength. |
| `composer` | `examples/aichat/src/main.rs`, `composer := GlassPanel` | Panel substrate overlay | Tint/decorative input substrate plus halo. | Phase 1 makes preset-driven; native mode disables halo and lowers decoration. |
| `ToolbarGlass` | `examples/aichat/src/main.rs`, `let ToolbarGlass = GlassPanel` | Panel substrate overlay | Small repeated glass toolbar. Static decoration today. | Keep as local panel exception for Phase 1; tune in Phase 3 after native substrate is visible. It is not a Phase 2 blocker because it is small and not full-window. |
| Sidebar separator | `examples/aichat/src/main.rs`, `SolidView { width: 1 draw_bg.color: #xEAD8B81E }` | Local readability surface | Thin translucent separator. | Keep. Does not block native substrate visibility. |
| Chat user card | `examples/aichat/src/main.rs`, `User := RoundedView show_bg: true color: #x0B2A22E6` | Local readability surface | Deliberately high alpha for message readability. | Keep for Phase 2. Consider native-mode alpha alternative in Phase 3 only if it visually hides too much glass. |
| Chat assistant card | `examples/aichat/src/main.rs`, `Assistant := RoundedView show_bg: true color: #x0B2A22E6` | Local readability surface | Deliberately high alpha for message readability. | Keep for Phase 2. Consider native-mode alpha alternative in Phase 3 only if it visually hides too much glass. |
| Code block background | `examples/aichat/src/main.rs`, `CodeView editor.draw_bg.color: #x031510EE` | Local readability surface | Very high alpha code surface. | Keep. Code readability is more important than showing native glass through code blocks. |
| Markdown default quote/code/table colors | `widgets/src/markdown.rs`, theme-driven `quote_bg_color`, `code_color`, `draw_table_*` | Local readability surface | Widget defaults use theme highlight/shadow and table draw colors. | Keep for Phase 2; Phase 3 can add native-mode Markdown tuning if bright-wallpaper validation fails. |
| Markdown table background | `widgets/src/markdown.rs`, `draw_table_bg`, `draw_table_header_bg`, `draw_table_line` | Local readability surface | Table cells/header use opaque-ish dark colors in widget defaults. | Keep. Tables need structured contrast. Tune only if validation shows excessive substrate hiding. |
| PortalList chat list | `examples/aichat/src/main.rs`, `list := PortalList` | Accidental large opaque surface | No explicit background in aichat template. | No action. If a future PortalList default changes, ensure it stays transparent or alpha-aware. |
| Splash block container | `examples/aichat/src/main.rs`, `splash_block := View/SolidView` | Dynamic generated UI | Host container is transparent/structural, but generated body may paint opaque roots. | Use v1 Splash guard: runtime warn + degrade for obvious opaque roots. |
| Splash-generated root backgrounds | `widgets/src/splash.rs`, runtime-evaluated `runsplash` body | Dynamic generated UI | Unknown until LLM output is evaluated. | Phase 0 chooses runtime warn + degrade. Full static analysis and token-only theming are deferred. |
| `GlassPanel` shader fill | `widgets/src/glass_panel.rs`, `sdf.fill_keep(vec4(..., tint_alpha))` | Panel substrate overlay | Alpha controlled by instance `tint_alpha`; border/highlight/noise/halo separately affect perceived intensity. | Phase 1 routes aichat call sites through substrate-aware preset values. |

## Splash Guard Decision

v1 chooses:

```text
Runtime warn + degrade for obvious opaque Splash roots.
```

Chosen degrade option:

```text
Option A: clamp detected root fill alpha to <= 0.5 in native mode.
```

Initial detection scope:

- root-level `View` or `RoundedView` with `width: Fill` and large/full height plus opaque `draw_bg.color`
- raw fully opaque white/black root fills
- generated root surfaces that are likely to cover the entire visible app card

Required log detail:

```text
[liquid-glass] splash-opaque-root view=<View|RoundedView> color=<value> action=clamp-alpha
```

Deferred:

- full Splash static analysis
- token-only color enforcement
- LLM prompt enforcement beyond normal app-generation guidance

## Phase 2 Gate Status

Current gate result: **not passed yet**.

Reasons:

- Phase 1.5 native view visibility proof has not been completed.
- Splash guard is selected but not implemented.
- Phase 1 preset plumbing is in progress.

No current aichat root/window surface appears to be an immediate opaque blocker: pass clear, window transparency, and root body background are already transparent.

## Follow-Up Tasks

- Implement the selected Splash runtime guard before Phase 2 native substrate work.
- Re-run this audit after Phase 1 preset plumbing is complete.
- Add screenshots during Phase 1.5 magenta native-view proof.
