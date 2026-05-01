# aichat Liquid Glass Implementation Tasks

Source design: [AICHAT-LIQUID-GLASS-DESIGIN.md](AICHAT-LIQUID-GLASS-DESIGIN.md)

## Status

Planning spec. No implementation is implied by this document.

This plan intentionally separates aichat-local work from Makepad platform work. Phase 2 native glass must not start until both Phase 0 and Phase 1.5 pass their gates.

**Phase 1.5 timeline is not under aichat's control.** It depends on Makepad framework review cycle. Phase 0 and Phase 1 work must not stall waiting for Phase 1.5 to land. Plan parallel branches accordingly.

## Objective

Implement aichat Liquid Glass as one Makepad UI with interchangeable substrates:

- default cross-platform shader glass
- optional macOS native `NSGlassEffectView` substrate
- future shader-backdrop blur/refraction substrate

The first shippable milestone is not "native glass exists in code". The first milestone is:

```text
AICHAT_GLASS_BACKEND=shader behaves exactly like today,
and the codebase has a measured path to make native glass visible.
```

Native glass is considered successful only when it is visibly present behind Makepad content on supported macOS and the renderer does not cover it with accidental opaque pixels.

## Non-Goals

- No per-panel native `NSGlassEffectView` mapping.
- No runtime substrate switching in v1.
- No AppKit inactive overlay in v1.
- No ShaderBackdrop implementation in the native-glass milestone.
- No macOS native-only fork of aichat UI.
- No cargo feature gate for native glass availability.
- No raw `cargo run` UI validation; use Makepad Studio remote per repo policy.

## Future v2 Compatibility Notes

When runtime substrate switching becomes a v2 feature, the following must be designed:

- atomic insertion/removal of `NSGlassEffectView` without a frame flash
- atomic re-application of `GlassPanel` preset across all panel instances
- handling in-flight Makepad render passes during transition
- re-evaluating `window.isOpaque` and clear color
- reconciling `GlassAppearance` state with both AppKit view hierarchy and Makepad shader state

v1 startup-only deliberately avoids these. **v2 designers must not assume the v1 startup code path is extensible by simply wrapping it in a setter.** Runtime switching is a different architectural problem and should be designed independently.

## Workstream Overview

| Workstream | Scope | Can Start Now | Blocks |
|---|---|---:|---|
| W0 Opaque Pixel Audit | inventory and classify opaque surfaces | yes | Phase 2 |
| W1 aichat Config + Presets | `GlassAppearance`, env parsing, substrate-aware shader presets | yes | Phase 3 |
| W1.5 Makepad Window RFC/Proof | verify/modify macOS view hierarchy + visibility proof | yes | Phase 2 |
| W2 Native Substrate | runtime `NSGlassEffectView` insertion + result-source-of-truth | no | Phase 0 + Phase 1.5 |
| W3 Visual Tuning | native overlay preset tuning | after W1/W2 | release quality |
| W4 Inactive Tint | Makepad-only inactive appearance | after W3 | polish |
| W5 ShaderBackdrop | cross-platform real blur/refraction | later | independent future |

## Shared Definitions

### Appearance Model

The implementation should converge on this model:

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

`ShaderBackdropConfig` is a future placeholder. It must remain `None` during v1 native work.

### Environment Contract

```text
unset                -> ShaderOnly
shader               -> ShaderOnly
macos-native         -> MacosNative { Regular } if available, else ShaderOnly + warn
macos-native-clear   -> MacosNative { Clear } if available, else ShaderOnly + warn
auto                 -> MacosNative { Regular } if available, else ShaderOnly
```

Substrate is read at startup only. Settings UI must not expose runtime switching in v1.

### Native Substrate Result State Machine

Critical invariant: `GlassAppearance` must reflect the resolved state of the native substrate as reported by the platform layer, not just the optimistic config value at startup. See LG-2.7.

```text
State 1: class missing                  -> ShaderOnly + ShaderDefault
State 2: class exists, pre-flight       -> ShaderOnly + ShaderDefault + warn
         check fails (responds-to-
         selector / alloc returns nil /
         style unsupported)
State 3: instantiation succeeds, view   -> ShaderOnly + ShaderDefault + warn
         installed on a proofed
         hierarchy that has not been
         runtime-verified for this
         session
State 4: instantiation succeeds, view   -> MacosNative { style } + NativeOverlay
         installed on a proofed
         hierarchy
```

State 3 is the most subtle: never combine `ShaderOnly` substrate decision with `NativeOverlay` panel preset. That combination produces low-alpha panels with no underlying glass — a worse product state than full shader fallback.

State 4 means "installed on a hierarchy proofed by LG-1.5.3, with all pre-flight checks passing". It does **not** claim per-frame runtime visibility verification unless LG-2.7 Option A is chosen. The spec deliberately calls this "proofed", not "actually visible", to avoid an unkeepable promise.

### Result Transport: Platform → aichat

The native substrate result must be reported back from the platform layer to the aichat (app) layer. The current Makepad `CxOsOp` path (e.g. `CreateWindow`, `SetWindowVisuals`) is one-way (app → platform). A return channel must be defined as part of LG-1.5.2 RFC and implemented in LG-2.7. See LG-2.7 for the concrete options.

Without a result transport, aichat would have to assume the optimistic startup config value — which means `AICHAT_GLASS_BACKEND=macos-native` would unconditionally apply `NativeOverlay` panel alphas even when the platform fell back to `ShaderOnly`. That is the exact mismatch the State Machine is designed to prevent.

## Phase 0: Opaque Pixel Audit

### Goal

Prove that native glass will not be hidden by Makepad rendering once inserted.

### Ownership

Documentation plus targeted code inspection. Avoid visual redesign in this phase unless an opaque surface has an obvious local fix.

### Inputs

- `examples/aichat/src/main.rs`
- `widgets/src/glass_panel.rs`
- `widgets/src/markdown.rs`
- `widgets/src/splash.rs`
- `platform/src/draw_pass.rs`
- platform window transparency paths as needed

### Tasks

#### LG-0.1 Inventory Large Opaque Surfaces

Search and inspect across both `*.rs` (live DSL is embedded in `script_mod! { ... }` / `live_design! { ... }` / `startup()`) and any `*.live` files if present:

```text
pass.clear_color
window.transparent
show_bg: true
draw_bg.color
draw_bg +: { color: ... }
RoundedView backgrounds
PortalList item/root backgrounds
Markdown code/prose/diagram surfaces
Splash root output
Button/TextInput default backgrounds
GlassPanel internal shader bg path
```

**Audit table minimum coverage** (each row is one surface; failing to enumerate one of these categories invalidates the audit):

| Required category | Minimum entries |
|---|---|
| `pass.clear_color` | at least 1 |
| `window.transparent` | at least 1 |
| `show_bg: true` (across `.rs` + `.live`) | all occurrences |
| `draw_bg.color` explicit assignment | all occurrences |
| `RoundedView` / `View` / `SolidView` default backgrounds | all widget classes used by aichat |
| `GlassPanel` self shader (panel's own draw_bg) | 1 |
| Markdown rendering surfaces | code block / blockquote / table cell — one row each |
| Splash generator root template | 1 |
| `PortalList` scroll viewport background | 1 |

Output an audit table in a new or existing spec section:

```text
surface | file:line | category | current alpha behavior | native-mode strategy
```

#### LG-0.2 Classify Each Surface

Use the four categories from the design:

- accidental large opaque surface
- panel substrate overlay
- local readability surface
- dynamic generated UI

Do not remove local readability surfaces blindly. Code blocks, inputs, diagrams, and Markdown cards may remain more opaque than panels.

#### LG-0.3 Choose Splash Guard v1

Default decision for v1:

```text
Runtime warn + degrade for obvious opaque Splash roots.
```

The task is to define detection scope, not to implement full static Splash analysis.

Minimum detection target:

- generated root-level `RoundedView` / `View` with full-width or full-height opaque fill
- raw white/black opaque colors used as large roots
- warning path that leaves enough information for prompt/debug improvement

**Degrade action** (implementer chooses one; spec does not lock the implementation):

- Option A: clamp detected root fill alpha to ≤0.5 in native mode
- Option B: wrap the generated root view in a substrate-aware container that owns the background decision

**Explicitly out of v1 scope**:

- Splash static analysis
- LLM prompt injection
- token-only color enforcement (deferred to v2)

The log path must preserve identifying information (view type, location, color value) so prompt and Splash runtime can be improved over time.

#### LG-0.4 Audit Output Gate

Phase 0 passes when **all** of these are true:

- Every surface listed in the LG-0.1 minimum coverage table appears in the audit table with a category and a native-mode strategy.
- Every entry classified as "accidental large opaque surface" has either:
  - a merged commit reference fixing it, or
  - an explicit, written reason it is not a Phase 2 blocker (not just "planned fix").
- Splash guard strategy is written down with the chosen degrade option.
- Phase 2 risk section is updated if any opaque surface cannot be resolved before Phase 2 entry.

"Planned fix" alone does not pass the gate. Either the fix is in, or the entry must justify why it does not block Phase 2.

## Phase 1: Config and aichat GlassAppearance

### Goal

Add substrate-aware configuration and panel preset plumbing without platform-native glass.

This phase must preserve current default visuals.

### Files

- `examples/aichat/src/main.rs`
- possibly `examples/aichat/src/liquid_glass.rs` if the helpers become too large
- `examples/aichat/specs/**`

### Tasks

#### LG-1.1 Add Types

Add aichat-local types for:

- `GlassAppearance`
- `GlassSubstrate`
- `MacosGlassStyle`
- `GlassPanelPreset`
- optional placeholder `ShaderBackdropConfig`

Keep these local to aichat until platform API shape is proven.

#### LG-1.2 Parse `AICHAT_GLASS_BACKEND`

Add startup parsing:

```text
unset
shader
macos-native
macos-native-clear
auto
```

Unsupported or unknown values should log a warning and fall back to `ShaderOnly`.

Do not add a settings control yet.

#### LG-1.3 Add Availability Stub

Before Phase 2, native availability should be a stub:

```text
macOS: false until platform proof exists
non-macOS: false
```

This lets config behavior and fallback logic be tested without native APIs.

#### LG-1.4 Replace `apply_glass_opacity`

Current code has:

```rust
fn glass_opacity_values(slider: f64) -> GlassOpacity
fn apply_glass_opacity(&self, cx: &mut Cx, opacity: f64)
```

Target:

```rust
fn glass_opacity_values(slider: f64, preset: GlassPanelPreset) -> GlassOpacity
fn apply_glass_appearance(&self, cx: &mut Cx, appearance: GlassAppearance, slider: f64)
```

The default path must produce the same shader values as current `ShaderOnly`.

#### LG-1.5 Audit GlassPanel Call Sites for Decoration Knobs

`widgets/src/glass_panel.rs` already exposes the relevant instance params (`tint_alpha`, `border_alpha`, `highlight_strength`, `noise_strength`, `halo_strength`, etc.). **This task is an audit, not a widget extension.**

- Walk every aichat call site that instantiates or configures `GlassPanel`.
- Confirm each decoration param is either:
  - driven by `apply_glass_appearance`, or
  - a justified hardcoded value with a written reason.
- Where a call site hardcodes a value that should be substrate-aware, change the call site to flow through `apply_glass_appearance`. **Do not modify the widget.**
- Only if a Phase 3 target value cannot be expressed via existing instance params should the widget be extended — and that becomes a separate widget task with its own review.

Phase 1 passes this task when every aichat-side call site is either preset-driven or has a written exception.

#### LG-1.6 Unit Tests

Add or update tests for:

- env parsing
- unknown env fallback
- non-macOS `macos-native` fallback
- default unset backend is `ShaderOnly`
- shader preset matches existing alpha curve
- native preset values are lower and monotonic with the slider

### Acceptance

- `cargo check -p makepad-example-aichat`
- `cargo test -p makepad-example-aichat`
- `cargo build -p makepad-example-aichat --release`
- Studio release run with unset env looks unchanged.
- Studio release run with `AICHAT_GLASS_BACKEND=shader` looks unchanged.
- `AICHAT_GLASS_BACKEND=macos-native` does not crash before Phase 2; it logs fallback.

## Phase 1.5: Makepad macOS Window Hierarchy RFC and Proof

### Goal

Answer two questions:

1. **Hierarchy**: Can Makepad host a native substrate below its render view without breaking rendering or input?
2. **Visibility**: When a native view is inserted below the Makepad render view, is it actually visible through Makepad's transparent layers?

This is framework-level work. Treat it as a gate, not as a quick aichat tweak.

### Files to Inspect

- `platform/src/os/apple/macos/macos_window.rs`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/os/apple/macos/macos_app.rs`
- `platform/src/os/apple/metal.rs`
- `platform/src/window.rs`
- existing issue notes around macOS backdrop insertion

### Tasks

#### LG-1.5.1 Verify Current Hierarchy

Document the current macOS view structure:

```text
NSWindow
contentView
MTKView / Makepad render view
optional existing effect/backdrop view path
```

Answer explicitly:

- Is `MTKView` currently the `contentView`?
- Is it a child of a container?
- Where is transparency configured?
- Where is layer opacity configured?
- Where is the existing `NSVisualEffectView` path?
- Which `CALayer` properties (`isOpaque`, `backgroundColor`, `compositingFilter`) sit between the render view and any view below it?

#### LG-1.5.2 Write RFC

Create a small RFC/spec, likely:

```text
examples/aichat/specs/aichat-macos-native-substrate-rfc.md
```

It must cover:

- proposed container hierarchy
- API ownership: Makepad platform general API vs aichat-specific hook
- how the API is exposed to widgets/apps
- input passthrough expectations
- lifecycle ownership and cleanup
- relation to the existing `NSVisualEffectView` insertion bug
- fallback if maintainers reject general platform changes
- **`NSGlassEffectView.Style` integer mapping**:
  - confirmed `regular` and `clear` `NSInteger` values
  - source: macOS 26.0 SDK header file path, OR a minimal Swift demo program where `Style.regular.rawValue` and `Style.clear.rawValue` are printed
  - **do not** attempt to discover these via KVC or generic runtime reflection — enum cases are compile-time constants, not KVC-accessible
  - if SDK headers unavailable to the implementer, the Swift demo route is mandatory before Phase 2 can start

#### LG-1.5.3 Native View Visibility Proof

This is **not** "insert an inert view without crashing". It is "insert an inert view and **prove it is visible** behind Makepad content".

Procedure:

- Insert a plain inert `NSView` (not `NSGlassEffectView`) below the Makepad render view.
- Set its `layer.backgroundColor` to a vivid, easily distinguishable test color (recommended: `#FF00FF` magenta — unlikely to occur naturally in aichat UI).
- Run aichat with `pass.clear_color` alpha = 0.
- Capture a screenshot via Studio remote run or macOS Screenshot.

Acceptance proof:

- The magenta test color is **clearly visible** through transparent Makepad regions in the screenshot.
- Makepad content still draws correctly on top.
- Clicks and text input still go to Makepad.
- Window resize keeps the native view aligned.
- No old native view leaks after rerun.

If the magenta is not visible, Phase 1.5 fails. The same root cause that hides magenta will hide `NSGlassEffectView` in Phase 2. Investigate Makepad layer opacity, MTKView `framebufferOnly`, `CAMetalLayer.isOpaque`, and contentView background before declaring Phase 1.5 done.

This proof is the predictive oracle for Phase 2 success.

#### LG-1.5.4 Gate Decision

Phase 1.5 passes only when one of these is true:

1. A general Makepad native substrate API is accepted **and** LG-1.5.3 visibility proof passes.
2. An explicit aichat-only hook is approved as temporary, **and** LG-1.5.3 visibility proof passes.
3. Native substrate is declared blocked. In this case:
   - aichat current `ShaderOnly` experience is preserved as already shipped
   - ShaderBackdrop plan (Phase 5) is escalated to next iteration
   - other aichat work is not blocked
   - user-facing documentation states clearly: "liquid glass currently has only a shader mode on this platform"
   - Phase 2 / Phase 3 / Phase 4 are formally cancelled, not paused

Option 3 is a fully terminal end-state, not a hanging crisis.

### Acceptance

- RFC/proof document exists.
- The proof has screenshots or Studio validation notes showing the test color visibly behind Makepad content.
- The chosen path identifies exact platform files to change.
- `NSGlassEffectView.Style` integer values are documented with source.
- Phase 2 can start without guessing the window hierarchy or style enum values.

## Phase 2: macOS Native Substrate

### Prerequisites

Required:

- Phase 0 passed (LG-0.4 gate).
- Phase 1 passed (Acceptance section).
- Phase 1.5 passed with a concrete platform strategy and a passing visibility proof.

Do not start this phase before those gates pass.

### Goal

Create startup-only native macOS glass substrate behind Makepad content, using runtime Objective-C class lookup, with a verified result reporting path.

### Likely Files

- `platform/src/os/apple/macos/macos_window.rs`
- `platform/src/os/apple/macos/macos.rs`
- `platform/src/window.rs`
- `examples/aichat/src/main.rs`

### Tasks

#### LG-2.1 Add Platform Data Model

Add minimal platform-facing types only after Phase 1.5 decides the API shape.

Keep it startup-only:

```text
Window created with substrate config.
No runtime set_native_glass(...) API in v1 unless the RFC requires it internally.
```

#### LG-2.2 Runtime Class Lookup

Use Objective-C runtime class lookup:

```rust
AnyClass::get(c"NSGlassEffectView")
```

Do not depend on compile-time `NSGlassEffectView` typed bindings. Do not introduce a cargo feature flag for native glass.

If class is missing:

- log fallback (machine-parseable)
- platform layer reports State 1 to aichat (see LG-2.7)
- do not fail startup

#### LG-2.3 Create and Insert Native View

Insert:

```text
NSGlassEffectView below Makepad render view
```

Rules:

- native view receives no input
- Makepad remains the only UI owner
- render view preserves alpha
- window remains non-opaque
- no `NSWindow` styleMask modifications
- no titlebar re-enabling
- no `NSWindow` subclassing

**Failure handling — pre-flight checks, not exception catching.**

Objective-C exceptions raised by `objc::msg_send!` cannot be reliably caught from Rust. `catch_unwind` does not handle ObjC exceptions, and any NSException reaching Rust frames is undefined behavior. Therefore:

- All failure modes must be detected via **pre-flight checks** before sending the selector. ObjC exceptions are treated as implementation bugs, not as a recoverable State 2.

Pre-flight checklist before each `msg_send!` that could raise:

| Check | Method | Failure → |
|---|---|---|
| class exists | `AnyClass::get(c"NSGlassEffectView")` returns `Some` | State 1 if `None` |
| class responds to `+alloc` | `instancesRespondToSelector:` for `init` etc. | State 2 |
| `alloc/init` returns non-nil | check pointer after `msg_send!` | State 2 if nil |
| instance responds to `setStyle:` | `respondsToSelector:` | State 2 |
| style integer in known range | check against LG-1.5.2 RFC values | State 2 |
| view created can be added to hierarchy | check superview after `addSubview:` | State 3 |

If any check fails, log the structured reason (LG-2.8) and report the corresponding state through the result transport (see LG-2.7 / Result Transport).

Any uncaught ObjC exception reaching Rust is a **bug**, not a normal State 2 fallback. It must be fixed by adding the missing pre-flight check, not by wrapping the call site in error recovery.

#### LG-2.4 Support Styles

Map env to style integer (using values established in LG-1.5.2 RFC):

```text
macos-native       -> Regular  (NSInteger value from RFC)
macos-native-clear -> Clear    (NSInteger value from RFC)
auto               -> Regular when available
```

The integer values must be sourced from LG-1.5.2 RFC, not invented at implementation time.

#### LG-2.5 Integrate with aichat Appearance

Use the result state from LG-2.7 to decide `GlassAppearance`. Do not assume class lookup success implies visibility.

```text
LG-2.7 State 1 -> ShaderOnly + ShaderDefault
LG-2.7 State 2 -> ShaderOnly + ShaderDefault + warn
LG-2.7 State 3 -> ShaderOnly + ShaderDefault + warn
LG-2.7 State 4 -> MacosNative { style } + NativeOverlay
```

**Critical**: never combine `ShaderOnly` substrate with `NativeOverlay` preset. That mismatch produces panels tuned for native glass without any glass underneath — worse than pure shader fallback.

#### LG-2.7 Define Native Substrate Result Source of Truth

(LG-2.6 reserved for any platform-side polish; LG-2.7 is the substrate result contract.)

The platform layer must report one of four states to aichat after substrate creation, and aichat must apply the corresponding `GlassAppearance` only:

| State | Trigger | aichat response |
|---|---|---|
| 1: class missing | `AnyClass::get` returns `None` | `ShaderOnly` + `ShaderDefault`, info log |
| 2: pre-flight check fails | `respondsToSelector:` false / `alloc/init` returns nil / style unsupported | `ShaderOnly` + `ShaderDefault`, warn log |
| 3: instantiation succeeds, view installed but not yet runtime-verified | view exists, hierarchy attach succeeded, but Option A self-check (if enabled) has not yet run or failed | `ShaderOnly` + `ShaderDefault`, warn log |
| 4: instantiation succeeds, view installed on proofed hierarchy | all pre-flight checks pass; hierarchy proven by LG-1.5.3 | `MacosNative { style }` + `NativeOverlay` |

**Visibility / proof-trust options for v1** (decided during LG-1.5.2 RFC):

- Option A: runtime self-check on first frame — walk `CALayer` chain from window root to native substrate; require all intermediate layers to have `isOpaque == false` and `backgroundColor == nil` (or transparent). This is the only option that can claim "actual visible state" semantics. Failure → State 3.
- Option B: trust LG-1.5.3 static proof and skip runtime self-check; treat State 3 as unreachable in normal build. State 4 means "installed on a proofed hierarchy", not "verified visible this session".
- Option C (recommended for v1): no per-frame runtime check; rely on LG-1.5.3 proof + Studio visual validation as the trust source. State 3 is only reached if LG-2.3 pre-flight checks detect a hierarchy attach failure. State 4 means "installed on a proofed hierarchy".

If Option B or C is chosen, the State 4 contract is **"proofed", not "verified"**. The spec is intentionally honest about this — claiming runtime visibility verification when none happens would create an unkeepable promise.

Whichever option is chosen, it must be specified before Phase 2 implementation starts. State 3 must not become a silent passthrough to State 4.

#### LG-2.7.A Result Transport Implementation

The platform layer must communicate the resolved state back to aichat. The current `CxOsOp` enum carries app → platform requests only (e.g. `CreateWindow`, `SetWindowVisuals`); there is no symmetric platform → app result for substrate creation.

**Choose one transport, document the choice in LG-1.5.2 RFC, implement here:**

- **Option T1 (recommended): event-based.**
  Add a new `Event` variant such as `WindowNativeSubstrateResolved { window_id, state: NativeSubstrateState }`. Platform layer dispatches it once per window after substrate creation completes (success or fallback). aichat handles it in its `handle_event` to (re-)apply `GlassAppearance` and panel preset.

- **Option T2: pull-based query.**
  Add a query API `Cx::native_substrate_state(window_id) -> NativeSubstrateState`. aichat polls it on first frame. Simpler than T1 but creates a frame-ordering hazard: aichat must not apply `NativeOverlay` before the query is consulted.

- **Option T3: stored-on-window.**
  Platform writes the resolved state into `CxWindow` and posts a generic redraw signal. aichat reads it during normal frame processing. Lowest API surface change but mixes platform and config state in `CxWindow`.

Required invariant regardless of transport choice: `GlassAppearance` cannot be finalized to `MacosNative { style }` until the resolved state has been observed. Until then, aichat must apply `ShaderOnly + ShaderDefault` as the optimistic-but-safe default.

Failure path: if the resolved state is never reported (platform bug, dropped event), aichat must keep `ShaderOnly + ShaderDefault` indefinitely rather than upgrading to `NativeOverlay`. A startup-time timeout warning is acceptable but not required.

#### LG-2.8 Result Logging Contract

Platform layer logs must use a stable structured tag so users and CI can grep:

```text
[liquid-glass] state=1 reason=class-missing
[liquid-glass] state=2 reason=alloc-failed detail=...
[liquid-glass] state=3 reason=visibility-unverified detail=...
[liquid-glass] state=4 substrate=macos-native style=regular
```

This is required for the failure-path validation matrix (Validation Policy section).

### Acceptance

- Supported macOS with native class **and** LG-2.7 State 4: native substrate is visible behind content.
- Unsupported macOS or LG-2.7 State 1/2/3: app starts and falls back to shader mode with `ShaderDefault` preset (no `NativeOverlay` mismatch).
- Linux/Windows compile without macOS glass code.
- `AICHAT_GLASS_BACKEND=shader` never creates native glass.
- `AICHAT_GLASS_BACKEND=auto` chooses native only when LG-2.7 State 4 is reached.
- Runtime switching is not exposed.
- Structured logs are emitted per LG-2.8.
- UI validation uses Studio release run.

## Phase 2.5: Window Shape Sync

### Goal

Fix visual leaks only if native substrate appears square under a rounded visual shell.

### Tasks

- verify whether aichat's borderless window is visually rectangular or rounded
- if rounded, synchronize native substrate corner radius
- inspect safe-area/titlebar extension only if glass visibly stops too early

### Acceptance

- no square native glass leak is visible behind a rounded aichat shell
- no new fullscreen/titlebar scope is pulled into v1

## Phase 3: Native Overlay Tuning

### Goal

Tune Makepad `GlassPanel` decoration so it complements system material instead of fighting it.

### Files

- `examples/aichat/src/main.rs`
- `widgets/src/glass_panel.rs` only if a Phase 3 target value cannot be expressed via existing instance params (rare; see LG-1.5)

### Tasks

#### LG-3.1 Apply Decoration Intensity Table

Native overlay target values:

| Element | NativeOverlay |
|---|---:|
| `app_shell.tint_alpha` | ~0.30 |
| `sidebar.tint_alpha` | ~0.38 |
| `main_area.tint_alpha` | ~0.34 |
| `composer.tint_alpha` | ~0.44 |
| border opacity | 0.6 scale |
| top highlight band | 0.4 scale |
| noise intensity | 0.3 scale |
| halo | off by default |

Do not tune only alpha. Border, highlight, noise, and halo must all be substrate-aware.

#### LG-3.2 Readability Surface Tuning

Review:

- Markdown code blocks
- assistant/user cards
- input composer
- generated Splash app container
- diagram/table backgrounds

Goal: native glass visible, text still readable on bright wallpaper.

#### LG-3.3 Slider Behavior

The existing `Glass` slider remains a panel opacity/readability control.

It must:

- affect `ShaderDefault`
- affect `NativeOverlay`
- not pretend to be native blur radius

### Acceptance

- bright wallpaper screenshot: text readable
- dark wallpaper screenshot: glass still visible
- no double-highlight artifacts
- `Glass` slider works in shader and native modes
- generated Splash content does not hide the whole substrate

## Phase 4: Inactive Tint

### Goal

Dim inactive aichat windows using Makepad-only panel adjustment.

### Tasks

- subscribe to `Event::WindowGotFocus` and `Event::WindowLostFocus` (already emitted by Makepad on macOS — see `platform/src/event/event.rs`)
- on `WindowLostFocus`: apply inactive multiplier `0.7` to all `GlassPanel` instances in the affected window
- on `WindowGotFocus`: restore the active preset
- multiplier applies on top of the substrate-aware preset (i.e. multiply each panel's `tint_alpha`, `border_alpha`, etc. by `0.7`, then apply); not a separate preset
- do not add AppKit overlay view
- do not add new platform events; existing `WindowGotFocus` / `WindowLostFocus` already cover macOS, Linux, and Windows

### Acceptance

- active window uses normal preset
- inactive window visibly dims
- focus toggling between two aichat windows: each independently switches preset
- input/hit testing unchanged
- no new native views added
- no new platform events introduced

## Phase 5: ShaderBackdrop Future Plan

### Goal

Split cross-platform real blur/refraction into an independent future plan.

### Tasks

- split existing v3 plan into:
  - `aichat-liquid-glass-v3-shader-backdrop.plan.md`
  - `aichat-liquid-glass-v3-macos-native.plan.md` if needed
- preserve current v3 shader-backdrop research
- keep it independent from `NSGlassEffectView`

### Acceptance

- native macOS plan and shader-backdrop plan are not conflated
- Linux/Windows path remains shader-only until ShaderBackdrop is implemented

## Validation Policy

### Static

Run after code phases:

```text
cargo check -p makepad-example-aichat
cargo test -p makepad-example-aichat
cargo build -p makepad-example-aichat --release
```

### Runtime

Use Makepad Studio remote release run for UI validation. Do not use raw `cargo run` for runtime UI verification.

### Substrate × Env Matrix

| Platform | Env | Expected |
|---|---|---|
| macOS without native class | unset | ShaderOnly |
| macOS without native class | `auto` | ShaderOnly |
| macOS without native class | `macos-native` | ShaderOnly + warn |
| macOS with native class | unset | ShaderOnly |
| macOS with native class | `shader` | ShaderOnly |
| macOS with native class | `auto` | MacosNative Regular (LG-2.7 State 4) |
| macOS with native class | `macos-native` | MacosNative Regular (LG-2.7 State 4) |
| macOS with native class | `macos-native-clear` | MacosNative Clear (LG-2.7 State 4) |
| Linux/Windows | unset | ShaderOnly |
| Linux/Windows | `macos-native` | ShaderOnly + warn |

### Failure-Path Matrix

| Failure mode | Expected behavior |
|---|---|
| class lookup returns None on old macOS | LG-2.7 State 1, structured log, no crash |
| class exists but `alloc/init` returns nil | LG-2.7 State 2, structured log, ShaderOnly+ShaderDefault, no crash |
| class exists, view created, but layer attach fails | LG-2.7 State 3, structured log, **ShaderOnly + ShaderDefault** (not NativeOverlay), no crash |
| `AICHAT_GLASS_BACKEND=garbage` | warn log, ShaderOnly fallback |
| System "Reduce Transparency" accessibility option enabled | system auto-degrades NSGlassEffectView; aichat must remain readable; verify no Makepad-side override breaks system degrade |
| Wallpaper change during session | system updates glass material automatically; verify no Makepad-side caching prevents update |
| Enter native fullscreen | v1 known limitation; glass may disappear or render incorrectly; **must not crash**; document in user-facing release notes |
| Stage Manager (if available on target macOS) | smoke test only; document any visual artifacts as known v1 limitations |
| Multiple displays with different backing scale factors | verify no glass artifacts on display change |

### Visual Cases

- bright wallpaper
- dark wallpaper
- generated Splash app with an opaque root (verify LG-0.3 degrade applies)
- active and inactive window
- resize near minimum size
- multiple displays
- Stage Manager if available
- fullscreen transition (expected v1 limitation, documented)
- Reduce Transparency on/off

## Commit Plan

Recommended commit boundaries:

1. `docs: add liquid glass implementation task spec`
2. `docs: add liquid glass opaque pixel audit`
3. `aichat: add glass appearance config and shader defaults`
4. `docs: add macos window hierarchy analysis (LG-1.5.1)`
5. `docs: add macos native substrate rfc (LG-1.5.2)`
6. `platform: prove inert native substrate visibility (LG-1.5.3)`
7. `platform: add startup native glass substrate with result state machine`
8. `aichat: tune native glass panel overlay`
9. `aichat: add inactive glass tint`

Do not batch Phase 1.5 RFC and Phase 1.5 proof in one commit. RFC review cycle is independent from proof implementation.

Do not batch Phase 1.5 proof and Phase 2 in one commit. If native insertion fails visually, rollback must not lose the proof and RFC groundwork.

## Open Questions

- Should the platform API be generic, such as `WindowSubstrate`, or specifically glass-oriented?
- Does Makepad currently set `MTKView` as `contentView`, or is there already a container suitable for insertion?
- Can the existing `NSVisualEffectView` path be repaired and reused structurally?
- Should Splash v1 guard prefer Option A (clamp alpha) or Option B (substrate-aware container wrap)?
- For LG-2.7 visibility verification, is Option A (runtime layer-chain self-check) worth the complexity for v1, or does Option C (trust LG-1.5.3 proof) suffice?
- For LG-2.7.A result transport, T1 (event), T2 (query), or T3 (stored-on-window)? T1 is recommended; final choice belongs to LG-1.5.2 RFC.

Resolved (no longer open):

- `NSGlassEffectView.Style` integer values — must be resolved in LG-1.5.2 RFC before Phase 2 starts.
- Inactive window event source — `Event::WindowGotFocus` / `Event::WindowLostFocus` already exist (`platform/src/event/event.rs`); Phase 4 subscribes to them.

## First Action

Start three parallel branches of work:

```text
Branch A: Phase 0 audit document.
Branch B: Phase 1 aichat config and preset plumbing.
Branch C: Phase 1.5 hierarchy fact-finding (LG-1.5.1) — feeds the RFC.
```

Branch C does not require Makepad maintainer review yet; it is read-only fact collection that unblocks LG-1.5.2 RFC drafting once Phase 0 / Phase 1 are landing.

Do not start native AppKit insertion until LG-1.5.3 visibility proof passes with the magenta test color clearly visible behind Makepad content.

## Current Implementation Status

As of the `aichat-liquid-glass-phase1` branch:

- Phase 0 audit and Phase 1 appearance plumbing are implemented.
- LG-1.5.3 proof path passes in a real macOS window: `AICHAT_NATIVE_SUBSTRATE_PROOF=magenta` is visible behind Makepad content when launched without `--stdin-loop`.
- Studio framebuffer runs cannot validate AppKit-native substrates. Native substrate validation must use the external-window runnable.
- `makepad-example-aichat-native` launches a real macOS window for proof testing.
- `makepad-example-aichat-macos-native` launches a real macOS window with `AICHAT_GLASS_BACKEND=macos-native` and clears the magenta proof override.
- Platform Phase 2 has an initial runtime lookup path for `NSGlassEffectView`. On the current validation machine the class is unavailable, so the expected result is a structured State 1 log and no crash:
  `[liquid-glass] state=1 reason=class-missing detail=NSGlassEffectView`.
- LG-2.7.A uses T1 event transport. The platform emits `Event::WindowNativeSubstrateResolved(WindowNativeSubstrateResolvedEvent)` after startup substrate creation. aichat starts in ShaderOnly and only switches to `MacosNative + NativeOverlay` when the event reports `WindowNativeSubstrateState::Installed`.
- Explicit native requests therefore fall back to ShaderOnly on old macOS without ever applying `NativeOverlay` against a missing native substrate.
