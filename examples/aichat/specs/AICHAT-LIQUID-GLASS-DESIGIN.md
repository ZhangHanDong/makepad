# aichat Liquid Glass Design

## Intent

aichat should keep one Makepad UI and support multiple glass substrates:

- **Shader glass**: the current cross-platform Makepad-rendered glass panels.
- **macOS native substrate**: optional native system glass behind the Makepad UI.
- **Future shader backdrop glass**: cross-platform real blur/refraction using offscreen passes.

The goal is not to maintain separate UIs. The goal is to keep the same aichat layout, widgets, Markdown, Splash, and app-generation behavior while swapping or combining the visual substrate under the same Makepad-rendered content.

## Current State

The current aichat implementation already has the right window-level preconditions for glass:

```text
window.transparent: true
pass.clear_color: #00000000
window.macos: MacosWindowConfig{chrome: MacosWindowChrome.Borderless}
```

The visible glass panels are Makepad shader panels:

```text
app_shell  -> GlassPanel
sidebar    -> GlassPanel
main_area  -> GlassPanel
composer   -> GlassPanel
toolbar    -> GlassPanel-derived ToolbarGlass
```

`widgets/src/glass_panel.rs` currently provides:

- translucent tint
- border
- rounded shape
- top highlight band
- subtle animated noise
- optional halo

It does **not** provide:

- native macOS Liquid Glass
- real desktop backdrop blur
- real refraction displacement
- per-panel system material

So the current implementation is best described as:

```text
Makepad shader glass panels over a transparent borderless window
```

## Design Principle

Native API and Makepad shader should be combined, not treated as two separate UI mechanisms.

Recommended stack:

```text
top:    Makepad UI content
        - text
        - buttons
        - Markdown
        - Splash-generated apps

middle: Makepad GlassPanel shader
        - emerald tint
        - panel shape
        - border
        - highlight
        - noise

below:  glass substrate
        - none / transparent window
        - macOS NSGlassEffectView
        - future offscreen blurred backdrop texture

bottom: desktop / window background / scene
```

This mirrors Ghostty's core lesson: native glass works only when the renderer cooperates by not painting an opaque base over the native material. The renderer and the native view are complementary.

## Glass Model

Use one appearance object and keep native style attached to the native substrate:

```rust
pub struct GlassAppearance {
    pub substrate: GlassSubstrate,
    pub backdrop: Option<ShaderBackdropConfig>,
}

pub enum GlassSubstrate {
    ShaderOnly,
    MacosNative { style: MacosGlassStyle },
}

pub enum MacosGlassStyle {
    Regular,
    Clear,
}

pub enum GlassPanelPreset {
    ShaderDefault,
    NativeOverlay,
    BackdropOverlay,
}
```

`GlassSubstrate` controls what exists below the Makepad render surface.

`GlassPanelPreset` controls how strongly Makepad panels tint and decorate the content above that substrate.

`ShaderBackdropConfig` is reserved for the future cross-platform real backdrop/refraction path. In v1 it is always `None`.

The same `GlassPanel` widget is used in all modes.

## Backend Semantics

### 1. ShaderOnly

Default mode. Works on macOS, Linux, Windows, and other Makepad targets.

```text
Makepad GlassPanel shader
transparent window where supported
no native system material
no real backdrop blur
```

This remains the default because it is stable, cross-platform, testable, and does not depend on macOS native glass availability.

### 2. MacosNative

Optional macOS-only native substrate.

```text
Makepad UI
Makepad GlassPanel shader overlay
NSGlassEffectView underneath Makepad render view
desktop / other windows
```

The native substrate provides the real system material. Makepad still owns panel shape, tint, border, highlight, layout, and interaction.

This mode must be availability-gated through runtime Objective-C class lookup rather than a cargo feature or SDK-bound typed API.

If unavailable, it falls back to `ShaderOnly`.

### 3. ShaderBackdrop

Future cross-platform real glass path.

This corresponds to the existing v3 plan:

```text
scene pass -> horizontal blur -> vertical blur -> GlassPanel samples blurred texture
```

This should not be blocked by macOS native work. It is a separate substrate that provides cross-platform real backdrop blur/refraction using Makepad rendering.

## Configuration and Lifetime

Initial configuration is environment-based:

```text
AICHAT_GLASS_BACKEND=shader
AICHAT_GLASS_BACKEND=macos-native
AICHAT_GLASS_BACKEND=macos-native-clear
AICHAT_GLASS_BACKEND=auto
```

Recommended behavior:

```text
unset                -> ShaderOnly
shader               -> ShaderOnly
macos-native         -> MacosNative { Regular } if available, else ShaderOnly + warn
macos-native-clear   -> MacosNative { Clear } if available, else ShaderOnly + warn
auto                 -> MacosNative { Regular } if available, else ShaderOnly
```

The existing `Glass` slider remains a panel opacity/readability control. It should not be reinterpreted as native blur radius.

Substrate is determined at startup in v1. Runtime switching is out of scope for v1.

This is intentional. Runtime switching requires atomically solving:

- insertion/removal of `NSGlassEffectView`
- `window.isOpaque` changes
- Makepad pass/layer alpha preservation
- substrate-aware panel alpha reapplication
- avoiding a visible flash during view hierarchy changes

Future UI can expose this in Settings only after runtime switching is designed and validated.

## Opaque Pixel Audit

Native glass is only visible if Makepad does not repaint an opaque surface over it.

Ghostty has a single base-background uniform to clear. aichat has many Makepad widgets, each with its own potential `draw_bg`. Therefore, aichat needs an explicit Opaque Pixel Audit before native substrate work is trusted.

The audit is a classification pass, not a mandate to remove every opaque pixel.

| Category | Examples | Strategy |
|---|---|---|
| Accidental large opaque surfaces | root view, scroll viewport, full-window `View { show_bg: true }` | Must be alpha-aware or disabled in native mode |
| Panel substrate overlay | `GlassPanel` for shell/sidebar/main/composer | Use substrate-aware preset |
| Local readability surfaces | code blocks, Markdown cards, input fields, selection, diagrams | Keep, but provide native-mode alpha alternatives |
| Dynamic generated UI | Splash/runsplash app content | Runtime guard or theme-token contract |

Minimum audit entry points:

- `pass.clear_color`
- root `body` background
- `GlassPanel.draw_bg`
- Markdown prose/code/diagram card backgrounds
- PortalList viewport/reuse items
- Button/TextInput default `draw_bg`
- all `show_bg: true` uses in aichat
- Splash-generated root backgrounds

### Splash Guard

Splash-generated UI is the hardest class because LLM output cannot be fully audited before runtime.

v1 must choose one guard:

| Option | Description | Tradeoff |
|---|---|---|
| A. Root filter | Wrap or post-process Splash root background to enforce substrate-aware alpha | Strong host control, may surprise generated apps |
| B. Theme token contract | Prompt LLM to use `theme`/host tokens instead of raw hex colors; token resolver applies alpha | Clean long term, requires stronger Splash prompt discipline |
| C. Runtime warn + degrade | Detect obvious opaque root fills and log/warn or render with fallback style | Lowest effort, weaker guarantees |

Recommended v1 choice: **C**, with a path to **B** once generated app theming is stabilized.

## Makepad Window Hierarchy RFC

macOS native substrate is not an aichat-only small API. It likely requires Makepad framework-level window hierarchy work.

Ghostty owns an AppKit/SwiftUI container and can insert:

```text
TerminalViewContainer
+-- NSGlassEffectView
+-- terminalView
```

Makepad may currently use the platform render view as the `window.contentView` or as a direct primary content view. If so, native glass requires a container:

```text
NSWindow contentView
+-- container NSView
    +-- NSGlassEffectView
    +-- Makepad render view / MTKView
```

This raises framework questions:

- Should Makepad accept a general native substrate API for all apps?
- Should `contentView` become a container view where it is not already?
- Should the existing `NSVisualEffectView` insertion bug be fixed in the same path?
- If framework-level hierarchy changes are rejected, is an aichat-specific hook acceptable despite being more brittle?

Phase 2 is blocked on a Phase 1.5 RFC/proof for this hierarchy. Do not start native substrate implementation assuming it is a small app-local call.

## Runtime Class Lookup Strategy

Do not copy Ghostty's Swift compile-time guard model mechanically.

Swift needs `#if compiler(...)` and `@available(...)` because the Swift type system requires the native type to exist at compile time. Rust + Objective-C runtime can be more flexible.

Preferred strategy:

```rust
fn try_create_glass_view(style: MacosGlassStyle) -> Option<Retained<AnyObject>> {
    let cls = AnyClass::get(c"NSGlassEffectView")?;
    let view: Retained<AnyObject> = unsafe { msg_send_id![cls, new] };
    unsafe {
        let _: () = msg_send![&view, setStyle: style.to_nsinteger()];
    }
    Some(view)
}
```

Benefits:

- no cargo feature gate needed for availability
- no minimum AppKit SDK type binding needed
- one binary works on old and new macOS
- missing class naturally falls back to `ShaderOnly`

This still remains macOS-only code through `#[cfg(target_os = "macos")]`, but the glass API itself should be runtime-discovered.

## macOS Native Substrate

### Target Structure

The target window structure is:

```text
NSWindow contentView
+-- NSGlassEffectView       native glass substrate
+-- Makepad render view     transparent-backed content
```

The Makepad render view must preserve alpha. The window must not be opaque.

Required platform behavior:

```text
window.isOpaque = false
window.backgroundColor = near-clear
Makepad clear color alpha = 0
Makepad root/background does not repaint an opaque surface
```

Ghostty uses a near-clear white background instead of fully clear. aichat can follow that if it improves visual matching:

```text
backgroundColor = white alpha 0.001
```

### View Ordering

Correct ordering is critical.

The native glass view must be below the Makepad render view. It should not receive input. Makepad remains responsible for all hit testing and UI behavior.

This is the main risk area because existing aichat notes already mention a platform insertion bug around macOS backdrop view ordering.

### Titlebar

aichat currently uses a borderless window and a custom in-app top bar. Native traffic-light controls must not be reintroduced.

Native glass should extend under the custom chrome if possible, but this is secondary. The first version can apply glass to the content area under the entire borderless window.

## GlassPanel Role

`GlassPanel` remains the primary aichat visual component.

It should not know whether the substrate is native, shader-only, or future backdrop blur. Instead, aichat applies different panel presets:

```rust
fn apply_glass_appearance(cx, appearance: GlassAppearance, slider: f64)
```

Responsibilities:

- compute panel tint alphas
- compute border opacity
- compute top highlight strength
- compute noise strength
- compute halo strength
- preserve readability
- keep current `Glass` slider behavior

### Decoration Intensity Table

Native mode needs more than lower alpha. It also needs weaker decoration so Makepad highlights do not fight the system material.

| Element | ShaderDefault | NativeOverlay |
|---|---:|---:|
| `app_shell.tint_alpha` | ~0.66 | ~0.30 |
| `sidebar.tint_alpha` | ~0.78 | ~0.38 |
| `main_area.tint_alpha` | ~0.70 | ~0.34 |
| `composer.tint_alpha` | ~0.82 | ~0.44 |
| border opacity | 1.0 scale | 0.6 scale |
| top highlight band | 1.0 scale | 0.4 scale |
| noise intensity | 1.0 scale | 0.3 scale |
| halo | allowed | off by default |

The exact values are tuning constants, but the substrate-aware curve is mandatory.

## Why Not Per-Panel Native Views

Do not map every `GlassPanel` to a separate `NSGlassEffectView` in the first version.

That would require:

- synchronizing Makepad widget rects to AppKit subviews
- matching z-order
- clipping to scroll views
- handling rounded corners per panel
- handling hit testing
- handling PortalList reuse
- handling animated layout changes

This would create a second UI layout system. It conflicts with the goal of keeping Makepad as the single UI owner.

The correct first step is window-level native substrate plus Makepad shader overlays.

## Inactive Window Tint

Ghostty uses an AppKit overlay above the native glass view. aichat should not start there.

| Option | Location | Pros | Cons |
|---|---|---|---|
| AppKit overlay | Between native glass and Makepad view | Closest to Ghostty | More native views, event passthrough risk |
| Makepad full-window overlay | Inside Makepad UI | Single UI layer | Needs careful hit behavior |
| Per-panel alpha adjustment | Makepad panel preset | Simplest | May shift colors |

Recommended v1: **per-panel alpha adjustment** on window resign/become key.

Example:

```text
inactive panel alpha = active panel alpha * 0.7
```

Upgrade to AppKit overlay only if panel alpha tuning is visually inadequate.

## Out of Scope for v1

The following are explicitly out of scope for v1 native substrate:

- runtime substrate switching
- per-panel native glass views
- native fullscreen behavior
- Stage Manager-specific behavior
- multi-display transition behavior
- window corner radius synchronization
- safe-area/titlebar glass extension
- AppKit inactive tint overlay
- ShaderBackdrop blur/refraction pass

These should be tracked as v2+ work, not treated as v1 bugs.

## Relationship to Existing Specs

### `aichat-liquid-glass-ui.spec.md`

This remains the product-level visual contract.

One boundary should be amended:

```text
Native macOS glass is allowed only as an optional platform substrate behind
the Makepad-rendered UI. It must not replace Makepad widgets, layout, Markdown,
Splash, or app interaction behavior.
```

### `aichat-liquid-glass-v2.plan.md`

This remains the shader-only/default plan.

Its output is still valuable as the fallback and as the Makepad overlay even in native mode.

### `aichat-liquid-glass-v3.plan.md`

This should be treated as the future cross-platform shader-backdrop path, not as the macOS native path.

Recommended split:

```text
aichat-liquid-glass-v3-shader-backdrop.plan.md
aichat-liquid-glass-v3-macos-native.plan.md
```

## Ghostty Differences

Ghostty should guide the native substrate design, but it cannot be copied directly.

| Dimension | Ghostty | aichat implication |
|---|---|---|
| Content surface | one terminal surface | many widgets and panels |
| Opaque entry point | one renderer uniform | distributed widget audit |
| Text/content | terminal cells | Markdown, rich text, Splash |
| Window shape | mostly system titlebar/window | borderless custom chrome |
| SDK guard | Swift compile/runtime guards | Rust runtime class lookup |
| View hierarchy | AppKit container exists | Makepad hierarchy may need RFC |
| Inactive tint | AppKit overlay | Makepad-only v1 |

## Implementation Phases

### Phase 0: Opaque Pixel Audit

Can run in parallel with Phase 1. Blocks Phase 2.

Work:

- inventory large opaque backgrounds
- classify each entry into the four audit categories
- choose Splash guard strategy
- define native-mode alpha alternatives for readability surfaces

Acceptance:

- list of opaque surfaces exists
- each surface has a substrate-mode strategy
- Splash guard strategy is chosen

### Phase 1: Config and GlassAppearance

Can run in parallel with Phase 0. Does not touch platform code.

Files:

- `examples/aichat/src/main.rs`
- `examples/aichat/specs/**`

Work:

- add `GlassAppearance`
- parse `AICHAT_GLASS_BACKEND`
- implement startup-only substrate selection
- keep default as `ShaderOnly`
- update `apply_glass_opacity` into substrate-aware `apply_glass_appearance`

Acceptance:

- current shader appearance remains unchanged by default
- `AICHAT_GLASS_BACKEND=shader` behaves exactly like current build
- `macos-native` on unsupported platforms falls back to `ShaderOnly` with warning

### Phase 1.5: Makepad macOS Window Hierarchy RFC and Proof

Blocks Phase 2.

Work:

- verify current macOS `contentView` / render view hierarchy
- propose container view structure if needed
- decide whether native substrate API belongs in Makepad platform generally
- fix or explicitly route around existing `NSVisualEffectView` insertion bug
- prove an inert native view can sit below Makepad render view without input or draw regressions

Acceptance:

- framework strategy is written down
- minimal proof shows a native view can be inserted below Makepad content
- no assumption remains that Phase 2 is a small app-local API

### Phase 2: macOS Native Substrate

Startup-only in v1.

Files likely involved:

- `platform/src/os/apple/macos/**`
- `widgets/src/window.rs` or nearby window handle/config plumbing
- `examples/aichat/src/main.rs`

Work:

- runtime lookup `NSGlassEffectView`
- create native glass substrate below Makepad render view
- support `Regular` and `Clear` style
- preserve window/layer alpha
- fall back to `ShaderOnly` if unavailable

Acceptance:

- on supported macOS, native glass is visible behind Makepad content
- on unsupported macOS, app logs fallback and remains usable
- Linux/Windows builds do not compile macOS native glass code
- runtime switching is not supported and not exposed

### Phase 2.5: Window Shape Sync

Optional.

Work:

- synchronize `cornerRadius` if aichat window shape needs it
- consider safe-area/titlebar extension

Acceptance:

- no square native glass leaks behind a rounded visual shell

### Phase 3: Native Overlay Tuning

Files:

- `examples/aichat/src/main.rs`
- `widgets/src/glass_panel.rs` if new instance params are needed

Work:

- tune alpha, border, highlight, noise, and halo using the decoration intensity table
- reduce decorative scene intensity where it competes with native material
- keep readability surfaces readable

Acceptance:

- text remains readable on bright and dark wallpapers
- native glass is visible but not visually noisy
- no double-highlight artifacts
- `Glass` slider still updates all panel alphas

### Phase 4: Inactive Tint

Makepad-only v1 strategy.

Work:

- detect window key/resign-key status
- apply inactive `GlassPanelPreset` multiplier

Acceptance:

- inactive aichat window visibly dims without adding native overlay views

### Phase 5: Future ShaderBackdrop

This is the cross-platform real glass path from the existing v3 plan.

Work:

- split the existing v3 plan into shader-backdrop and macOS-native plans
- scene pass
- blur passes
- `GlassPanel` samples blurred texture
- optional refraction displacement

Acceptance:

- Linux/Windows/macOS shader mode gets real blurred backdrop without native APIs

## Testing and Validation

Static checks:

```text
cargo check -p makepad-example-aichat
cargo test -p makepad-example-aichat
cargo build -p makepad-example-aichat --release
```

Runtime UI validation must use Makepad Studio remote per repo policy.

Validation matrix:

```text
macOS, native class unavailable:
  auto -> ShaderOnly
  macos-native -> ShaderOnly + warn
  macos-native-clear -> ShaderOnly + warn

macOS, native class available:
  unset -> ShaderOnly
  shader -> ShaderOnly
  auto -> MacosNative { Regular }
  macos-native -> MacosNative { Regular }
  macos-native-clear -> MacosNative { Clear }

Linux/Windows:
  unset -> ShaderOnly
  shader -> ShaderOnly
  auto -> ShaderOnly
  macos-native -> ShaderOnly + warn
```

Additional runtime cases:

- old macOS launch
- supported macOS launch
- bright wallpaper
- dark wallpaper
- generated Splash app with opaque root
- active/inactive window
- resize near minimum size
- multiple displays
- Stage Manager if available
- fullscreen transition, expected v1 behavior documented

Visual checks:

- no native traffic-light controls
- borderless drag region still works
- root background is not opaque
- native glass visible behind content in native mode
- text/cards remain readable over bright wallpaper
- `Glass` slider visibly changes panel opacity
- generated Splash content does not accidentally hide the whole substrate

## Risks

| Risk | Probability | Impact | Mitigation | Phase |
|---|---:|---:|---|---|
| Opaque pixels cover native glass | High | Native mode appears broken | Phase 0 audit | 0 |
| Makepad window hierarchy cannot accept substrate | Medium | Phase 2 blocked | Phase 1.5 RFC/proof | 1.5 |
| Old SDK typed binding fails | Medium | Build failure | Runtime class lookup | 2 |
| Decoration conflicts with system highlights | Medium | Visual artifacts | Decoration intensity table | 3 |
| Splash app breaks glass contract | Medium | Generated app hides substrate | Runtime guard / theme-token path | 0 |
| Rounded visual shell mismatches native glass | Low | Visual leak | Phase 2.5 shape sync | 2.5 |
| Fullscreen native glass behavior differs | Low | Glass disabled or odd in fullscreen | v2 explicit handling | v2 |
| Runtime switching flashes | Avoided | N/A | startup-only v1 | 1 |

## Recommended Next Step

Start Phase 0 and Phase 1 in parallel:

```text
Phase 0: Opaque Pixel Audit
Phase 1: GlassAppearance + startup env config + substrate-aware presets
```

Do not start Phase 2 until Phase 0 and Phase 1.5 are complete.

The first platform-facing work should be a Makepad macOS window hierarchy RFC/proof, not a direct aichat-only `NSGlassEffectView` insertion.
