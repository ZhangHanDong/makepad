# aichat Liquid Glass Phase F Audit

## Status

Phase F is partially landed. The current branch now has a reusable semantic
layer for the main glass controls and aichat surfaces, while native interactive
AppKit/UIKit controls remain intentionally deferred.

This audit is based on source inspection after commit `25072c86`.

## Implemented Semantic Widgets

Source: `widgets/src/glass_panel.rs`.

| Widget | Base | Current role |
|---|---|---|
| `GlassSeparator` | `View` | Horizontal material-aware separator. |
| `GlassVSeparator` | `View` | Vertical material-aware separator. |
| `GlassToolbar` | `GlassPanel` | Makepad-rendered toolbar grouping; no nested native container. |
| `GlassButton` | `ButtonFlat` | Regular glass-aware button. |
| `GlassIconButton` | `GlassButton` | Compact square icon/action button. |
| `GlassPrimaryButton` | `GlassButton` | Prominent commit/send button. |
| `GlassNavButton` | `GlassButton` | Sidebar navigation row button. |
| `GlassComposer` | `GlassPanel` | Input composer surface; may opt into native descriptors. |
| `GlassSidebar` | `GlassPanel` | Persistent navigation surface; may opt into native descriptors. |
| `GlassCard` | `RoundedView` | Message/readability surface; no per-message native panels. |

## aichat Adoption

Source: `examples/aichat/src/main.rs`.

| aichat node | Current semantic widget | Notes |
|---|---|---|
| `ToolbarGlass` | `GlassToolbar` | Local sizing and colors remain app-specific. |
| `PillButton` | `GlassButton` | Used by composer pill actions such as `clear_button`. |
| `IconButton` | `GlassIconButton` | Used by `attach_button`, `mention_button`, and `tools_button`. |
| `SendButton` | `GlassPrimaryButton` | Keeps the local circular send-button shader. |
| sidebar nav rows | `GlassNavButton` | Covers sidebar navigation and settings controls. |
| `User` / `Assistant` templates | `GlassCard` | Preserves rounded readability card behavior. |
| `sidebar` | `GlassSidebar` | Keeps native descriptor settings; one of the four native panels. |
| `composer` | `GlassComposer` | Keeps native descriptor settings; one of the four native panels. |
| sidebar/main divider | `GlassVSeparator` | Replaces the raw vertical `SolidView` divider. |

## Remaining ButtonFlat Exceptions

These remain app-local `ButtonFlat` users:

- `cancel_button`
- message `copy_button`
- message `delete_button`

Recommended follow-up:

- `GlassNavButton` is implemented for sidebar navigation rows.
- Add a quieter `GlassUtilityButton` variant for copy/delete/cancel controls.
- Keep message utility buttons Makepad-rendered; do not make them native
  interactive controls in v4.1/v4.2.

## GlassPanel Exceptions

These remain direct `GlassPanel` users in aichat:

- `app_shell`
- `main_area`

Current classification:

- Keep both direct for now. They are structural native panels, not controls.
- They are part of the four-panel AppleNativeUnderlay proof and should not be
  converted to per-control semantic components until a `GlassShell` /
  `GlassMainSurface` naming step is explicitly scoped.

## Other Local Glass Controls

- `GlassSlider` remains an app-local `SliderMinimal` style.
- This is acceptable for Phase F because slider behavior is not part of the
  Apple native Liquid Glass API surface, but it should eventually become a
  reusable glass-aware control token if more apps need it.

## Native Control Boundary

Phase F does not implement native `NSButton` / `UIButton.Configuration` glass
controls. Current `GlassButton`, `GlassToolbar`, `GlassIconButton`, and
`GlassPrimaryButton` are Makepad-rendered and sit above native underlay panels.

Native interactive controls require a separate hit-test and event-forwarding
phase. They must not be inferred from the existence of these semantic aliases.

## Completion Assessment

Phase F is sufficient to proceed to the next major route work when:

- `GlassNavButton` and `GlassUtilityButton` are either implemented or explicitly
  deferred.
- `app_shell` and `main_area` direct `GlassPanel` usage is either renamed into
  structural semantic surfaces or documented as permanent low-level native
  panel usage.
- The v4 spec status table is updated from "Phase F-I future work" to reflect
  the landed Phase F semantic layer.

Until those are done, Phase F should be considered mostly implemented but not
fully closed.
