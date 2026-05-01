# aichat macOS Native Substrate RFC

Source task: [AICHAT-LIQUID-GLASS-IMPLEMENTATION-TASKS.md](AICHAT-LIQUID-GLASS-IMPLEMENTATION-TASKS.md), Phase 1.5.

## Status

Draft RFC and code-grounded hierarchy audit.

Phase 1.5 is **not passed** yet. The magenta native-view proof path is implemented behind an environment flag, but it has not been validated through Studio screenshots.

## Goal

Allow a future macOS `NSGlassEffectView` substrate to sit below Makepad-rendered aichat content while Makepad remains the single UI owner for layout, drawing, input, Markdown, and Splash.

The proof target is intentionally not `NSGlassEffectView` first. It is:

```text
plain NSView with magenta layer background
inserted below Makepad render content
visible through transparent Makepad regions
without breaking input or resize
```

If magenta is not visible, native glass will not be visible either.

## Current macOS Hierarchy

Current code path:

```text
platform/src/os/apple/macos/macos_window.rs
MacosWindow::alloc_window
  self.view = get_macos_class_global().view alloc

MacosWindow::init
  [self.view initWithPtr: ...]
  [self.window setContentView: self.view]
  [self.window makeFirstResponder: self.view]

platform/src/os/apple/macos/macos.rs
MetalWindow::new
  ca_layer = [CAMetalLayer new]
  [ca_layer setDelegate: cocoa_window.view]
  [view setWantsLayer: YES]
  [view setLayer: ca_layer]
```

Current structure:

```text
NSWindow
└── contentView = MakepadView
    └── layer = CAMetalLayer
```

`MakepadView` is the render/input view. It is not currently a child of an AppKit container.

## Transparency Points

Window transparency is configured in:

```text
platform/src/os/apple/macos/macos_window.rs
MacosWindow::set_window_visuals
  [window setOpaque: NO] when visuals.transparent
  [window setBackgroundColor: clearColor] when visuals.transparent
```

Metal layer transparency is configured in:

```text
platform/src/os/apple/macos/macos.rs
Cx::handle_platform_ops / CreateWindow
  [ca_layer setOpaque: NO] when visuals.transparent
  [ca_layer setBackgroundColor: CGColor(..., alpha=0.0)] when visuals.transparent

Cx::handle_platform_ops / SetWindowVisuals
  same layer opacity/background update
```

aichat currently sets:

```text
pass.clear_color: #00000000
window.transparent: true
body.draw_bg.color: #00000000
```

These are the correct window-level preconditions for a native substrate.

## Existing Effect View Path

Current `WindowBackdrop` implementation is in:

```text
platform/src/os/apple/macos/macos_window.rs
MacosWindow::set_window_visuals
```

It creates `NSVisualEffectView` and inserts it as:

```text
[self.view addSubview: effect_view positioned: 0i64 relativeTo: nil]
```

This is the fragile path referenced by the aichat liquid-glass specs. It inserts the effect view into `MakepadView`, not below a separate Makepad render view inside a container.

This path should not be copied for `NSGlassEffectView`. It is useful as evidence that the existing architecture lacks a clean substrate layer.

## Problem

For native glass, the desired structure is:

```text
NSWindow
└── contentView = MakepadContainerView
    ├── native substrate view
    │   └── NSGlassEffectView or proof NSView
    └── MakepadView
        └── layer = CAMetalLayer
```

The current structure cannot express "below MakepadView but above the window background" because `MakepadView` itself is the `contentView`.

Inserting native views inside `MakepadView` is the wrong ownership model:

- z-order is coupled to Makepad's render/input view
- effect/backdrop views compete with a Metal-backed layer
- future per-view input and resize behavior becomes ambiguous
- the existing `NSVisualEffectView` insertion path is already known brittle

## Proposed Architecture

Introduce a platform-owned container view for standard macOS windows:

```text
MacosWindow {
    window: NSWindow
    container_view: NSView        // new
    view: MakepadView             // existing render/input view
    native_substrate_view: id     // optional, nil unless requested/proofing
}
```

Startup hierarchy:

```text
[window setContentView: container_view]
[container_view addSubview: native_substrate_view]  // optional, below
[container_view addSubview: view]                   // Makepad render/input, above
[window makeFirstResponder: view]
```

Both child views must autoresize with the container:

```text
NSViewWidthSizable | NSViewHeightSizable
```

`MakepadView` remains first responder and continues to own input.

## API Ownership

Recommended ownership: **Makepad platform general API**, not an aichat-only hook.

Reasoning:

- the hierarchy problem exists at `MacosWindow`, not at aichat
- current `WindowBackdrop` already tries to expose cross-app platform substrate behavior
- future apps may need the same "native view below render view" slot
- result reporting belongs near `WindowVisuals` / window creation, not inside aichat widgets

Suggested platform concept:

```rust
pub enum WindowNativeSubstrate {
    None,
    MacosGlass { style: MacosNativeGlassStyle },
    MacosProofColor { rgba: Vec4f }, // debug/proof only, cfg/test gated if desired
}
```

The exact public API can be renamed, but it should model a window substrate, not a panel effect.

## Result Transport

Recommended transport: **T1 event-based**.

Add a platform-to-app event after substrate startup resolution:

```rust
Event::WindowNativeSubstrateResolved {
    window_id: WindowId,
    state: NativeSubstrateState,
}
```

Reasoning:

- `CxOsOp` is currently app -> platform only
- aichat must not apply `NativeOverlay` until the platform reports State 4
- event ordering is easier to reason about than first-frame polling
- platform can report failures with structured reasons exactly once

Safe default before the event:

```text
ShaderOnly + ShaderDefault
```

If the event never arrives, aichat must stay in shader mode.

## Native Substrate State

The platform must report exactly one resolved startup state:

| State | Meaning | aichat response |
|---|---|---|
| 1 | native class missing | `ShaderOnly + ShaderDefault` |
| 2 | pre-flight check failed | `ShaderOnly + ShaderDefault` |
| 3 | view installed but hierarchy/visibility not trusted | `ShaderOnly + ShaderDefault` |
| 4 | view installed on proofed hierarchy | `MacosNative + NativeOverlay` |

State 3 must not silently upgrade to State 4.

For v1, recommended visibility trust option: **Option C** from the implementation task spec.

```text
No per-frame runtime self-check.
Trust LG-1.5.3 magenta proof + Studio visual validation.
State 4 means "installed on a proofed hierarchy", not "verified visible every session".
```

## Pre-Flight Checks

Objective-C exceptions must not be used as control flow. Any selector that can fail must be guarded before `msg_send!`.

Minimum checks:

- class exists: `AnyClass::get(c"NSGlassEffectView")`
- object allocation/init returns non-nil
- instance responds to `setStyle:`
- style integer is one of the RFC-documented values
- view has expected superview after insertion

Failure logs must use:

```text
[liquid-glass] state=<n> reason=<stable-reason> detail=<optional>
```

## Style Integer Mapping

Current local SDK check:

```text
xcrun --show-sdk-path
rg NSGlassEffectView <SDK>/System/Library/Frameworks/AppKit.framework/Headers
```

Result: no `NSGlassEffectView` headers found in the local SDK available to this workspace.

Therefore Phase 2 is blocked on one of:

- macOS 26 SDK headers that expose `NSGlassEffectView.Style`
- a minimal Swift demo compiled on a machine with the new SDK that prints:
  - `NSGlassEffectView.Style.regular.rawValue`
  - `NSGlassEffectView.Style.clear.rawValue`

Do not invent these integer values in Rust. Do not try to discover them via KVC.

## Magenta Visibility Proof

Proof view:

```text
NSView
wantsLayer = YES
layer.backgroundColor = #FF00FFFF
inserted below MakepadView inside container_view
```

Proof trigger:

```text
AICHAT_NATIVE_SUBSTRATE_PROOF=magenta
```

Current implementation:

```text
platform/src/os/apple/macos/macos_window.rs
  MacosWindow.container_view
  MacosWindow.proof_substrate_view
  MacosWindow::install_magenta_proof_substrate()

platform/src/os/apple/macos/macos.rs
  CreateWindow checks AICHAT_NATIVE_SUBSTRATE_PROOF=magenta
```

Required visual setup:

- aichat `pass.clear_color` alpha remains 0
- aichat `window.transparent` remains true
- root body background remains transparent
- proof color visible through the shell outside local readability surfaces

Acceptance:

- Studio screenshot clearly shows magenta through transparent Makepad regions
- Makepad content remains above the magenta
- click/type still target Makepad widgets
- resize keeps magenta aligned with the window
- rerun does not leak duplicate proof views

If magenta is not visible, investigate:

- `CAMetalLayer.isOpaque`
- `CAMetalLayer.backgroundColor`
- `MakepadView` layer opacity/background
- container view opacity/background
- pass clear alpha
- large opaque Makepad surfaces from the Phase 0 audit

## Relationship to Existing `WindowBackdrop`

The current `WindowBackdrop` implementation should either:

1. be migrated onto the new container substrate slot, or
2. remain unchanged while native glass uses the new slot.

Do not build new native glass on top of the current `self.view addSubview: positioned: 0i64` path.

If the existing `NSVisualEffectView` bug is fixed during this work, it should be a separate platform commit from aichat native glass behavior.

## Implementation Sketch

High-level platform steps:

1. Add `container_view: ObjcId` to `MacosWindow`.
2. Allocate/init `container_view` in `MacosWindow::alloc_window` / `init`.
3. Set `window.contentView = container_view`.
4. Add `self.view` as a child of `container_view`.
5. Keep `window.makeFirstResponder(self.view)`.
6. Move size/autoresize assumptions from content view to both child views.
7. Add proof substrate insertion under `self.view`.
8. Validate magenta visibility with Studio.
9. Only after proof passes, add runtime `NSGlassEffectView` lookup.

## Open Questions

- Should `container_view` be introduced for all macOS windows or only windows that request a native substrate?
- Should popups also use a container, or only standard windows?
- Should current `WindowBackdrop` be repaired in the same platform layer or deferred?
- What exact event payload type should carry native substrate result state?
- Where should structured substrate logs live: platform log only, or event payload plus log?
- Should the proof-color substrate be compiled only in debug/test, or kept as a hidden env/debug mode for future platform regressions?

## Gate Status

Phase 1.5 status: **not passed**.

Passed:

- current hierarchy inspected
- RFC strategy written
- local SDK checked for `NSGlassEffectView` headers
- container-view hierarchy implemented for macOS windows
- magenta proof-substrate insertion path implemented behind `AICHAT_NATIVE_SUBSTRATE_PROOF=magenta`

Blocked:

- magenta native-view visibility proof not validated through Studio screenshot
- Studio screenshot validation not run
- `NSGlassEffectView.Style` integer values unavailable in local SDK

Phase 2 must not start until those blockers are resolved.
