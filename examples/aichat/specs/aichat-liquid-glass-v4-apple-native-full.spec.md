# aichat Liquid Glass v4: Full Apple Native Support

## Status

Design route. No implementation is implied by this document.

This document supersedes the "single native substrate" mental model from v1 for future Apple native Liquid Glass work. The v1 work proved that Makepad can insert a native macOS glass substrate and reach State 4. v4 defines what is needed to make Makepad support Apple's full Liquid Glass model across macOS and iOS/iPadOS.

Related documents:

- [aichat-liquid-glass-v1-release-notes.md](aichat-liquid-glass-v1-release-notes.md)
- [aichat-liquid-glass-v3-macos-native.plan.md](aichat-liquid-glass-v3-macos-native.plan.md)
- [AICHAT-LIQUID-GLASS-OPAQUE-PIXEL-AUDIT.md](AICHAT-LIQUID-GLASS-OPAQUE-PIXEL-AUDIT.md)
- [aichat-liquid-glass-v3-shader-backdrop.plan.md](aichat-liquid-glass-v3-shader-backdrop.plan.md)

External references:

- Apple AppKit `NSGlassEffectView`: <https://developer.apple.com/documentation/appkit/nsglasseffectview>
- Apple AppKit `NSGlassEffectContainerView`: <https://developer.apple.com/documentation/appkit/nsglasseffectcontainerview>
- Apple UIKit `UIGlassEffect`: <https://developer.apple.com/documentation/uikit/uiglasseffect>
- Apple UIKit `UIGlassContainerEffect`: <https://developer.apple.com/documentation/uikit/uiglasscontainereffect>
- Apple SwiftUI `GlassEffectContainer`: <https://developer.apple.com/documentation/swiftui/glasseffectcontainer>
- Apple SwiftUI "Applying Liquid Glass to custom views": <https://developer.apple.com/documentation/SwiftUI/Applying-Liquid-Glass-to-custom-views>

Local reference:

- `/Users/zhangalex/Work/Projects/FW/robius/OpenSwiftUI/Sources/OpenSwiftUICore/Shape/ShapeStyle/Material.swift`
- `/Users/zhangalex/Work/Projects/FW/robius/OpenSwiftUI/Sources/OpenSwiftUICore/Graphic/BackdropEffect.swift`
- `/Users/zhangalex/Work/Projects/FW/robius/OpenSwiftUI/Sources/OpenSwiftUICore/Graphic/Color/VibrantColorStyle.swift`
- `/Users/zhangalex/Work/Projects/FW/robius/OpenSwiftUI/Sources/OpenSwiftUI/Layout/Separator/PlainDividerStyle.swift`

## Objective

Support Apple native Liquid Glass in Makepad as a first-class platform feature:

- macOS: `NSGlassEffectView` and `NSGlassEffectContainerView`
- iOS/iPadOS: `UIGlassEffect`, `UIGlassContainerEffect`, and `UIVisualEffectView`
- Makepad widgets: `GlassContainer`, `GlassPanel`, and glass-aware controls
- aichat: sidebar, main area, composer, toolbar controls, buttons, separators, popups, and generated Splash content can coexist with native glass

The desired end state is not "native glass is visible around the edge of the window." The desired end state is:

```text
Makepad layout produces semantic glass surfaces.
Apple platforms render those surfaces with native Liquid Glass.
Makepad retains readable text, input, Markdown, and generated UI.
Unsupported platforms fall back without app-specific forks.
```

## Non-Goals

- Do not replace Makepad's cross-platform shader glass.
- Do not implement ShaderBackdrop blur/refraction in this native phase.
- Do not expose AppKit/UIKit classes directly to app code.
- Do not require SwiftUI.
- Do not support arbitrary vector paths in the first native glass release.
- Do not place native views for every scrolling list row.
- Do not make runtime substrate switching part of the first full-native milestone.
- Do not mix `AppleNative` and `ShaderBackdrop` in the same window in v4.1.
- Do not put native glass panels above the Makepad Metal view in v4.1.
- Do not support interactive native glass panels in v4.1. All v4.1 native glass panels are input passthrough.

## Key Finding

Apple's Liquid Glass model is not a single backdrop view.

The API shape has three levels:

| Level | SwiftUI | macOS AppKit | iOS UIKit | Makepad equivalent needed |
|---|---|---|---|---|
| Single glass surface | `glassEffect(_:in:)`, `Glass` | `NSGlassEffectView` | `UIGlassEffect` inside `UIVisualEffectView` | `GlassPanel` native descriptor |
| Multi-surface container | `GlassEffectContainer` | `NSGlassEffectContainerView` | `UIGlassContainerEffect` | `GlassContainer` native descriptor |
| Semantic components | `GlassButtonStyle`, toolbar/presentation styles, foreground/material environment | AppKit controls and view hierarchy | UIKit controls and visual effect hierarchy | glass-aware Makepad widgets/theme |

Therefore v4 must add:

- native glass containers
- multiple native glass surfaces
- layout-to-native geometry sync
- shape/radius sync
- glass-aware foreground and component styling
- accessibility/fallback policies

## OpenSwiftUI Observations

OpenSwiftUI does not currently provide a complete Liquid Glass API implementation, but it shows useful architectural patterns.

### Material as Style, Not View

`Material.swift` models material as a `ShapeStyle`, not as a plain view. It resolves through environment and can affect descendant styling.

Makepad implication:

- `GlassPanel` should remain a widget, but "glassness" should also exist as a style/environment concept.
- Controls inside glass should not manually choose one-off colors.
- Theme resolution should know whether the current subtree is on glass.

### Background Material in Environment

OpenSwiftUI has `EnvironmentValues.backgroundMaterial`. Other styles read it.

Makepad implication:

- Add a glass/material environment state:

```rust
pub struct GlassEnvironment {
    pub backend: GlassBackend,
    pub material: GlassMaterial,
    pub active: bool,
    pub reduce_transparency: bool,
    pub increased_contrast: bool,
}
```

### Vibrancy

`VibrantColorStyle` applies foreground colors differently when a background material exists.

Makepad implication:

- Add semantic foreground tokens for glass:

```text
theme.color_on_glass_primary
theme.color_on_glass_secondary
theme.color_on_glass_tertiary
theme.color_on_glass_disabled
```

- Native glass mode should not rely on hard-coded aichat colors for all readability.

### Separators Adapt to Material

`PlainDividerStyle` and `SeparatorShapeStyle` adapt when `backgroundMaterial` exists.

Makepad implication:

- Add `GlassSeparator` or glass-aware separator styling.
- Border/separator color should derive from the glass environment, not from each app.

### Backdrop Is a Render Primitive

`BackdropEffect` is represented as scale, color, and filters. It is not a regular fill.

Makepad implication:

- Keep native glass and ShaderBackdrop separate backends.
- A future ShaderBackdrop path should use render pipeline primitives, not expand `GlassPanel` into ad hoc color math.

## Architecture

### Layer Model

```text
Makepad app / widgets
    |
    | GlassContainer / GlassPanel / GlassButton / GlassToolbar
    v
Glass descriptors collected after layout
    |
    | rect, radius, style, tint, hit-test, grouping, z-order
    v
Platform native glass manager
    |
    | macOS: NSGlassEffectContainerView + NSGlassEffectView
    | iOS: UIVisualEffectView + UIGlassContainerEffect + UIGlassEffect
    v
Apple native Liquid Glass
```

### Backend Model

```rust
pub enum GlassBackend {
    ShaderOverlay,
    AppleNative,
    ShaderBackdrop,
}
```

`ShaderOverlay` is the current cross-platform fallback. `AppleNative` owns AppKit/UIKit glass views. `ShaderBackdrop` remains a later cross-platform blur/refraction backend.

v4.1 permits only one active backend per window. `AppleNative` panels and `ShaderBackdrop` panels must not be mixed in the same window. This avoids two independent backdrop systems sampling/compositing the same visual area with incompatible assumptions.

### Compositing Model

v4.1 uses the same high-level compositing model proven by v1:

```text
NSWindow / UIWindow content root
    native glass container view(s)
        native glass panel view(s)
    transparent Makepad Metal view
        all Makepad-rendered content, text, controls, overlays, input handling
```

Rules:

- Native glass panels live below the transparent Makepad Metal view.
- Makepad content is always rendered above native glass in v4.1.
- `NativeGlassPanelDescriptor.z_order` only orders native glass panels relative to other native glass panels inside the same native container.
- `z_order` does not interleave native panels with Makepad draw order.
- `NativeGlassHitTest::Interactive` is not supported in v4.1. Descriptors that request it must be rejected or downgraded to `Passthrough` with a warning.
- Glass buttons and toolbars in v4.1 are Makepad-rendered controls placed on top of native glass panels.
- Multi-Metal-layer or multi-render-view composition is explicitly out of scope.

This is the only viable first implementation because it preserves Makepad's current event routing and avoids splitting the renderer into multiple transparent native views.

### Decisions for Phase B

These are design decisions, not implementation-time open questions:

1. Native glass export is opt-in.
   - `GlassPanel` defaults to `native: false`.
   - Only panels with `native: true` produce native descriptors.
   - This prevents per-row list items and existing apps from accidentally creating native views.
2. Descriptor collection is widget-tree driven after layout.
   - The widget knows its final rect and shape after layout.
   - App code should not manually register rects.
   - The platform receives a full post-layout `NativeGlassBatch`.
3. v4.1 native panel budget is 12 panels per window.
   - If a batch exceeds 12 visible native panels, the platform must reject the batch or downgrade the excess panels to shader fallback.
   - aichat's expected initial usage is 3-5 panels.
4. v4.1 shape support is rounded rect plus capsule.
   - Arbitrary paths are deferred.
5. v4.1 native panel hit testing is passthrough only.
   - Interactive native controls are a later GlassButton/native-control phase.
6. Descriptor geometry uses Makepad logical units.
   - `rect`, `radius`, and `spacing` are expressed in the same logical coordinate space as Makepad layout.
   - Platform backends convert to AppKit/UIKit coordinates and backing scale at the boundary.
7. A single `GlassContainer` may contain panels with different `NativeGlassStyle` values.
   - Apple native containers do not require one style per container.
   - Separate containers are for grouping/z-order/layout, not for style homogeneity.
8. v4.1 supports one native glass container per window.
   - This removes container ordering ambiguity for the first implementation.
   - Nested toolbar/control grouping remains Makepad-rendered until a later multi-container phase.
   - Panel `z_order` ordering is therefore well-defined within the single v4.1 container.

### Descriptor Model

The platform layer should receive a full batch of descriptors after layout. `WindowId` is the Makepad platform window identifier from `makepad_platform`.

```rust
pub struct NativeGlassBatch {
    pub window_id: WindowId,
    pub containers: Vec<NativeGlassContainerDescriptor>,
}

pub struct NativeGlassContainerDescriptor {
    pub id: LiveId,
    pub rect: DRect,
    pub spacing: f64,
    pub panels: Vec<NativeGlassPanelDescriptor>,
}

pub struct NativeGlassPanelDescriptor {
    pub id: LiveId,
    pub rect: DRect,
    pub shape: NativeGlassShape,
    pub style: NativeGlassStyle,
    pub tint: Option<Vec4>,
    pub hit_test: NativeGlassHitTest,
    pub z_order: i32,
    pub visible: bool,
}

pub enum NativeGlassShape {
    RoundedRect { radius: f64 },
    Capsule,
}

pub enum NativeGlassStyle {
    Regular,
    Clear,
}

pub enum NativeGlassHitTest {
    Passthrough,
    Interactive,
}
```

The batch model is important because containers need to reason about multiple surfaces together.

In v4.1, `containers.len()` must be `0` or `1`. Multi-container support is a later phase; the `Vec` shape is kept so the shared API does not need a breaking change when that phase starts.

All descriptor geometry values are Makepad logical units. `DRect` uses Makepad's logical coordinate space, not AppKit points, UIKit points, or physical pixels. The platform manager performs coordinate conversion, y-axis conversion where needed, and backing-scale handling before calling native APIs.

`NativeGlassStyle::Regular` and `NativeGlassStyle::Clear` map to the raw values validated in v3 on the current macOS 26 runtime:

```text
Regular -> 0
Clear   -> 1
```

### State Model

v1 keeps the compatibility log:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

In a multi-panel v4 window, the v1 compatibility log reports the container-level aggregate state. In a single-panel window, it keeps the v1 single-substrate meaning.

v4 adds batch/container/panel state:

```rust
pub struct NativeGlassBatchResult {
    pub window_id: WindowId,
    pub backend_state: NativeGlassBackendState,
    pub containers: Vec<NativeGlassContainerResult>,
}

pub struct NativeGlassContainerResult {
    pub id: LiveId,
    pub state: NativeGlassInstallState,
    pub reason: &'static str,
    pub installed_panels: usize,
    pub failed_panels: usize,
    pub panels: Vec<NativeGlassPanelResult>,
}

pub struct NativeGlassPanelResult {
    pub id: LiveId,
    pub state: NativeGlassInstallState,
    pub reason: &'static str,
}
```

Required log schema:

```text
[liquid-glass] native-container state=installed platform=macos containers=N panels_installed=M panels_failed=F
[liquid-glass] native-panel id=<id> state=<state> reason=<reason>
```

Container result states aggregate panel results. A failed non-required panel should not crash the app; it should fall back to shader rendering for that panel.

## Platform Requirements

### macOS

Use runtime Objective-C lookup for new classes/selectors:

- `NSGlassEffectContainerView`
- `NSGlassEffectView`
- `initWithFrame:`
- `setStyle:`
- `setTintColor:`
- `setCornerRadius:`
- `setContentView:`
- `setSpacing:` if present on `NSGlassEffectContainerView`

macOS manager responsibilities:

- create one native container view per Makepad `GlassContainer`
- create/update/remove descendant `NSGlassEffectView` instances
- keep native views below the transparent Makepad Metal view
- convert Makepad coordinates into AppKit view coordinates
- update frames on resize/layout change
- update corner radius on descriptor change
- map `GlassContainer.spacing` to native container spacing when the selector exists
- handle missing classes/selectors with structured fallback
- emit stable logs and platform events
- enforce v4.1 passthrough hit testing
- enforce the per-window panel budget
- implement `Capsule` by setting `cornerRadius = min(width, height) / 2`; there is no separate capsule selector

Validation target:

```text
[liquid-glass] native-container state=installed platform=macos containers=N panels_installed=M panels_failed=0
```

#### macOS Hit Test Policy

v4.1 all native glass views must be input passthrough.

The Makepad Metal view remains the input owner. AppKit native glass panels must not consume mouse events. `NativeGlassHitTest::Interactive` is reserved for a later native-control phase and must not be silently accepted by the v4.1 backend.

`NativeGlassHitTest::Interactive` exists for future API shape, but v4.1 must reject or downgrade it to `Passthrough`. There is no separate `interactive` boolean in the panel descriptor; hit-test policy is the single source of truth.

#### macOS Container Spacing

`GlassContainer.spacing` is part of Phase C, not Phase H. It maps to native container spacing/morph behavior where supported. Phase H may add animated spacing changes and transition choreography, but the static spacing semantics must exist with the container backend.

### iOS/iPadOS

Use UIKit visual effect hierarchy:

- `UIVisualEffectView`
- `UIGlassEffect`
- `UIGlassContainerEffect`

Intended minimum shape:

```swift
let containerEffect = UIGlassContainerEffect()
let container = UIVisualEffectView(effect: containerEffect)
container.frame = containerRect
container.isUserInteractionEnabled = false

let panelEffect = UIGlassEffect(style: .regular)
let panel = UIVisualEffectView(effect: panelEffect)
panel.frame = panelRectInContainer
panel.layer.cornerRadius = radius
panel.layer.masksToBounds = true
panel.isUserInteractionEnabled = false

container.contentView.addSubview(panel)
```

The exact UIKit initializer/property names must be validated against the target SDK in Phase A. This skeleton defines the intended hierarchy: one container effect view, with child visual effect views representing panels.

iOS manager responsibilities:

- create native visual effect views for Makepad glass surfaces
- support safe area and rotation
- support keyboard transitions without stale frames
- ensure touch passthrough in v4.1
- validate iPad Stage Manager and split view behavior after macOS is stable

iOS should not be implemented in the same PR as the first macOS native container work. The shared descriptor API should land first, then platform backends can be added independently.

## Makepad Widget Requirements

### GlassContainer

New widget concept:

```text
GlassContainer {
    native: true
    spacing: 20
    children...
}
```

Responsibilities:

- establishes a glass grouping scope
- maps `spacing` to native container merge/morph distance when supported
- collects descendant `GlassPanel` native descriptors
- maps to Apple native glass container when available
- falls back to shader grouping when unavailable

### GlassPanel

Existing `widgets/src/glass_panel.rs` should evolve from "shader panel only" to "semantic glass surface."

New properties:

```text
native: bool
native_style: NativeGlassStyle
native_hit_test: Passthrough | Interactive
shape: RoundedRect | Capsule
```

Defaults:

```text
native: false
native_style: Regular
native_hit_test: Passthrough
shape: RoundedRect
```

Existing `GlassPanel` users must remain behavior-compatible. If `native` is omitted, the widget behaves exactly like the current shader panel.

Native mode rendering split:

- Apple native backend renders material.
- Makepad keeps optional readability overlay, border, focus ring, and content.
- Shader-only fallback keeps current SDF fill/highlight/noise.

### GlassButton

Needed because Liquid Glass has interaction behavior, not just background.

Properties:

- regular/prominent variants
- interactive native glass
- hover/down/focus state
- disabled state
- icon + label support

Maps conceptually to SwiftUI `GlassButtonStyle` / `GlassProminentButtonStyle`.

Important distinction: native `GlassButton` is not the same mechanism as `UIGlassEffect` or `NSGlassEffectView` panels. On Apple platforms it should eventually use the platform's native button glass appearance/configuration where available. In v4.1 and v4.2, `GlassButton` remains Makepad-rendered on top of native glass panels.

The first implementation is a semantic ButtonFlat variant with glass-aware
foreground, fill, border, hover, down, and focus colors. Native interactive
controls are deferred until Makepad has an explicit AppKit/UIKit hit-test and
event-forwarding policy.

`GlassIconButton` and `GlassPrimaryButton` are Makepad-rendered variants of
`GlassButton` for compact icon actions and prominent commit/send actions. They
do not imply AppKit/UIKit button controls in v4.1/v4.2.

### GlassToolbar

For groups of buttons and small controls.

Responsibilities:

- remains Makepad-rendered in v4.1/v4.2
- exposes grouping spacing
- does not create nested native containers in the single-container phase
- defers native panel union/grouping to the multi-container phase
- keeps controls readable on bright and dark backgrounds

### GlassSidebar / GlassCard / GlassComposer

aichat needs these first:

- `GlassSidebar`: large vertical native glass panel with separators and nav buttons
- `GlassCard`: message card/readability surface, native glass optional
- `GlassComposer`: input surface with stronger readability and controls

`GlassSidebar` is a semantic Makepad navigation surface. It may opt into native
descriptors through normal `GlassPanel` properties, but nav controls remain
Makepad-rendered in v4.1/v4.2.

`GlassCard` is a semantic Makepad readability surface for messages and
generated content. The current implementation preserves rounded-card rendering
and does not create per-message native panels.

`GlassComposer` is a semantic Makepad surface for text input and composer
actions. It may opt into native descriptors through normal `GlassPanel`
properties, but it does not replace Makepad text input with a native AppKit or
UIKit text control in v4.1/v4.2.

### GlassSeparator

Separators must be material-aware. In native mode, separators should be subtle and high-contrast aware.

### GlassPopup / GlassModal

Defer until panel/container foundation is stable. Popups introduce z-order, focus, and platform-window edge cases.

## Theme and Environment

Add glass-aware theme tokens:

```text
theme.color_on_glass_primary
theme.color_on_glass_secondary
theme.color_on_glass_tertiary
theme.color_on_glass_disabled
theme.color_separator_on_glass
theme.color_glass_readability_tint
theme.color_glass_focus_ring
```

Add runtime environment inputs:

- color scheme
- active/inactive window state
- reduce transparency
- increased contrast
- platform native glass availability

Fallback policy:

| Condition | Behavior |
|---|---|
| Apple native API available | Use AppleNative descriptors |
| Apple native API missing | ShaderOverlay or ShaderBackdrop fallback |
| reduce transparency enabled on AppleNative | Let the system adjust native material; app strengthens foreground/separator tokens |
| reduce transparency enabled on shader fallback | Replace glass with high-contrast opaque/semi-opaque surfaces |
| increased contrast enabled | Strengthen foreground, separator, and focus tokens |
| inactive window | Makepad dimming first; native inactive overlay only in a later phase |

## aichat Integration Target

The first app target is aichat.

Initial native panel mapping:

| aichat node | Native glass role |
|---|---|
| `app_shell` | outer container/surface, optional |
| `sidebar` | native glass panel |
| `main_area` | native glass panel |
| `composer` | native glass panel, stronger readability overlay |
| toolbar controls | `GlassToolbar` or grouped `GlassButton` |
| nav buttons | `GlassButton` or material-aware button variant |
| separators | `GlassSeparator` |
| chat cards | fallback to local readability surfaces initially; see `User`/`Assistant` card entries in [AICHAT-LIQUID-GLASS-OPAQUE-PIXEL-AUDIT.md](AICHAT-LIQUID-GLASS-OPAQUE-PIXEL-AUDIT.md) |
| code blocks | keep high-contrast local background |
| Splash output | keep opaque root guard |

Success target:

- center of sidebar/main/composer shows native Liquid Glass, not only window edges
- native panels maintain correct rects through resize
- text remains readable on bright and dark wallpapers
- input, buttons, Markdown selection, and Splash events still work
- no mouse/touch interception unless explicitly interactive
- unsupported runtime falls back without crash

## Phased Route

### Phase A: API Matrix and Native Prototypes

Deliverables:

- `APPLE-LIQUID-GLASS-API-MATRIX.md`
- macOS prototype for one container with multiple glass views
- iOS prototype for one container with multiple glass visual effect views
- runtime logs proving class/selector availability

Acceptance:

- macOS validates `NSGlassEffectContainerView` and multiple `NSGlassEffectView` children
- iOS validates `UIGlassContainerEffect` and `UIGlassEffect`
- style/tint/hit-test/corner APIs are documented
- unsupported platforms fail closed to shader
- style/tint color conversion uses sRGB in v4.1, matching Makepad internal color assumptions

### Phase B: Shared Descriptor API

Deliverables:

- shared `NativeGlassBatch`
- shared descriptors and enums
- platform event/result types
- tests for diff semantics and fallback decisions
- batch validation for the 12-panel per-window budget
- downgrade/reject behavior for `NativeGlassHitTest::Interactive` in v4.1

Acceptance:

- descriptors are independent of aichat
- no AppKit/UIKit types leak into widget code
- unsupported platforms compile and ignore descriptors safely
- existing `GlassPanel` usage remains shader-only unless `native: true`

### Phase C: macOS Native Container Backend

Deliverables:

- runtime lookup for `NSGlassEffectContainerView`
- descriptor diff manager
- create/update/remove native panels
- frame/radius/style/tint sync
- passthrough hit testing
- static `GlassContainer.spacing` mapping to native container spacing/morph behavior
- per-container and per-panel install results

Acceptance:

- Studio release run reaches native container installed state
- multiple native panels are visible
- resize keeps panels aligned
- stale panels are removed
- State 4 style logs remain available
- container logs report installed and failed panel counts
- `NativeGlassHitTest::Interactive` descriptors do not consume AppKit input in v4.1

### Phase D: Widget Collection and DSL

Deliverables:

- `GlassContainer`
- `GlassPanel` native descriptor export
- minimal native shape support: rounded rect and capsule
- fallback to existing shader panel

Acceptance:

- a simple Makepad example can declare multiple native panels
- native descriptors update after layout
- shader fallback looks comparable to current UI

### Phase E: aichat Native Panels

Deliverables:

- wrap aichat shell in `GlassContainer`
- map sidebar/main/composer to native panels
- lower Makepad overlay further in native mode
- keep Markdown/code/readability exceptions
- keep Splash guard

Acceptance:

- direct visual inspection shows glass in panel interiors
- bright/dark wallpaper readability passes
- no input regression
- release build and Studio remote run pass

### Phase F: Glass-Aware Controls

Deliverables:

- `GlassButton`
- `GlassToolbar`
- `GlassSeparator`
- glass theme tokens
- hover/down/focus styling
- `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`, covering native `NSButton` / `UIButton` glass appearance and configuration options for later platform control integration

Acceptance:

- toolbar/nav/composer controls use semantic glass styling
- separators adapt to glass environment
- reduce transparency and increased contrast have explicit behavior
- native button glass configuration remains separate from panel descriptors
- `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md` exists and covers at least `NSButton` bezel/configuration options and `UIButton.Configuration` glass appearance options on the target SDK.

### Phase G: iOS/iPadOS Backend

Deliverables:

- UIKit backend for descriptor API
- safe area / rotation / keyboard sync
- touch passthrough

Acceptance:

- iOS example shows multiple native glass panels
- rotation and keyboard do not leave stale frames
- iPad split view and Stage Manager are smoke-tested

### Phase H: Advanced Native Behavior

Deliverables:

- animated spacing/morph transitions
- scroll edge glass
- fullscreen and multi-display handling
- native inactive-window behavior

Acceptance:

- advanced features are opt-in
- core aichat glass remains stable
- no regression in shader fallback

### Phase I: Popup and Modal Glass

Deliverables:

- popup/modal glass design for separate AppKit `NSPanel` / UIKit `UIWindow` surfaces
- focus and hit-test policy for transient native windows
- z-order policy across main window and transient windows

Acceptance:

- popup/modal glass does not regress main-window native panels
- modal focus and dismissal remain owned by Makepad semantics

## Risk Register

| Risk | Impact | Mitigation |
|---|---|---|
| Native views do not align with Makepad layout | Visual breakage | descriptor snapshots, resize tests, Studio screenshots |
| Native views intercept input | Broken UI | v4.1 passthrough only; reject/downgrade interactive descriptors |
| Multiple native panels are expensive | Performance regression | container batching, diff updates, 12 visible native panels per window |
| AppKit/UIKit API changes across beta/runtime | Incorrect selectors/styles | runtime lookup, env overrides, API matrix |
| Glass looks good but text is unreadable | Usability failure | glass-aware foreground tokens, contrast modes |
| Scroll/list native panels explode view count | Performance and z-order bugs | prohibit native per-row panels initially |
| iOS and macOS diverge too early | abstraction churn | land descriptor API before iOS backend |
| Shader fallback drifts | cross-platform regression | keep shader contract tests and visual smoke tests |

## Validation Matrix

| Area | macOS | iOS/iPadOS | Fallback |
|---|---|---|---|
| API availability | runtime class/selector logs | runtime class/property logs | no crash |
| one panel | native visible | native visible | shader visible |
| multiple panels | container merge visible | container merge visible | shader grouping |
| resize | rects stay aligned | rotation/safe area aligned | unaffected |
| input | passthrough works | touch passthrough works | unaffected |
| state reporting | container/panel counts logged | container/panel counts logged | fallback reason logged |
| accessibility | reduce transparency fallback | reduce transparency fallback | high contrast surface |
| aichat | sidebar/main/composer native | later | shader/default |

## Implementation Order Recommendation

Recommended first implementation plan:

```text
v4.1 / Phase A: Apple API Matrix and Native Prototypes
```

Scope:

- Apple API matrix
- macOS native prototype with one container and multiple panels
- iOS native prototype skeleton
- runtime class/selector validation logs
- no aichat integration yet

Then:

```text
v4.2 / Phase B: Shared descriptor API
v4.3 / Phase C: macOS native container backend
v4.4 / Phase D: Makepad GlassContainer / GlassPanel descriptor export
v4.5 / Phase E: aichat native panel integration
v4.6 / Phase F: glass-aware controls
v4.7 / Phase G: UIKit backend
v4.8 / Phase H: advanced native behavior
v4.9 / Phase I: popup/modal glass
```

This ordering avoids mixing API discovery, platform view management, widget DSL, aichat styling, and iOS backend in one branch.

## Current Branch Implementation Status

Status as of this branch:

| Phase | Status | Evidence |
|---|---|---|
| Phase A | Landed | `APPLE-LIQUID-GLASS-API-MATRIX.md`, `apple-liquid-glass-phase-a-runtime-notes.md`, Step 01 spec, commit `e94941df` |
| Phase B | Landed | shared `NativeGlassBatch` descriptors, validation tests, Step 02 spec, commit `d37d5a5a` |
| Phase C | Landed for macOS backend plumbing | `CxOsOp::SetNativeGlassBatch`, macOS installer, non-macOS no-op handlers, Step 03 spec, commit `b09029c7` |
| Phase D | Landed for widget descriptor export | `GlassContainer`, opt-in `GlassPanel.native`, widget collector tests, Step 04 spec, commit `77781499` |
| Phase E | Landed for aichat wiring | single `glass_container`, four native panels, substrate-gated native toggle, Step 05 spec, commit `9c776f3e` |
| Phase F-I | Future work | controls, UIKit backend, advanced behavior, and popup/modal glass remain unimplemented |

Studio visual validation remains pending. The branch has compile/unit-test
coverage for the Rust path, but native glass visibility still requires a
Makepad Studio release run on macOS 26.

## Remaining Open Questions

1. Should iOS wait until macOS aichat integration is visually accepted, or start after descriptor API lands?
2. Should the 12-panel budget be a hard error, a warning plus downgrade, or configurable per app?
