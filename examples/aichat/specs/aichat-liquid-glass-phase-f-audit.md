# aichat Liquid Glass Phase F Audit

## Status

Phase F is landed for Makepad-rendered semantic controls and structural glass
surfaces. Native interactive AppKit/UIKit controls remain intentionally
deferred.

This audit is based on current branch source inspection.

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
| `GlassUtilityButton` | `GlassButton` | Low-emphasis copy/delete/cancel button. |
| `GlassShell` | `GlassPanel` | Outer app shell surface; may opt into native descriptors. |
| `GlassMainSurface` | `GlassPanel` | Main content surface; may opt into native descriptors. |
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
| utility actions | `GlassUtilityButton` | Covers cancel, copy, and delete controls. |
| `app_shell` | `GlassShell` | Structural outer native panel; one of the four native panels. |
| `main_area` | `GlassMainSurface` | Structural main native panel; one of the four native panels. |
| `User` / `Assistant` templates | `GlassCard` | Preserves rounded readability card behavior. |
| `sidebar` | `GlassSidebar` | Keeps native descriptor settings; one of the four native panels. |
| `composer` | `GlassComposer` | Keeps native descriptor settings; one of the four native panels. |
| sidebar/main divider | `GlassVSeparator` | Replaces the raw vertical `SolidView` divider. |

## Remaining ButtonFlat Exceptions

No remaining app-local `ButtonFlat` exceptions are known in the hand-authored
aichat UI surface.

Recommended follow-up:

- `GlassNavButton` is implemented for sidebar navigation rows.
- `GlassUtilityButton` is implemented for copy/delete/cancel controls.
- Keep message utility buttons Makepad-rendered; do not make them native
  interactive controls in v4.1/v4.2.

## Direct GlassPanel Usage

No remaining app-local direct `GlassPanel` usage is known in the hand-authored
aichat UI surface. Structural panels now use `GlassShell`, `GlassMainSurface`,
`GlassSidebar`, and `GlassComposer`.

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

Phase F is closed for the current AppleNativeUnderlay route. Remaining native
button work belongs to a later explicit native interactive-control phase, not to
the Makepad-rendered semantic layer.
