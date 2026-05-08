# Apple Native Liquid Glass Completion Audit

## Objective Restatement

Objective: complete support for Apple native Liquid Glass APIs in Makepad/aichat.

Concrete success criteria:

1. macOS AppKit backend installs and updates native Liquid Glass containers and
   panels from shared descriptors.
2. UIKit backend installs and updates native Liquid Glass containers and panels
   from the same shared descriptors.
3. Makepad widgets expose semantic glass surfaces and controls without leaking
   platform-specific AppKit/UIKit types.
4. Native interactive controls have a defined hit-test and event ownership
   policy before they are enabled.
5. Advanced native behavior is handled or explicitly gated: runtime switching,
   fullscreen, Stage Manager, multi-display, inactive-window behavior, animated
   morph spacing, and popup/modal surfaces.
6. Full native interior Liquid Glass is either implemented through a proven
   Apple-native interleave architecture or explicitly not claimed for the
   current renderer.
7. Validation gates cover compile checks, descriptor tests, platform logs,
   Studio release runs, manual visual checks, and SDK/runtime availability.

Current conclusion: not complete.

## Prompt-To-Artifact Checklist

| Requirement | Current evidence | Status |
|---|---|---|
| Apple API inventory | `APPLE-LIQUID-GLASS-API-MATRIX.md`; `apple-liquid-glass-phase-a-runtime-notes.md` | Partial: official docs recorded, local SDK still lacks typed 26 symbols. |
| macOS AppKit backend | `platform/src/os/apple/macos/macos_window.rs`; Studio logs `state=4 substrate=macos-native style=clear style_raw=1`; `containers=1 panels_installed=4 panels_failed=0` | Landed for AppleNativeUnderlay. |
| Shared descriptors | `NativeGlassBatch`, `NativeGlassContainerDescriptor`, `NativeGlassPanelDescriptor` in `platform/src/event/window.rs` | Landed. |
| Descriptor validation | `validate_v4_1`, max 12 panels, single container, reject `NativeGlassHitTest::Interactive` | Landed for v4.1 constraints. |
| Makepad widget export | `GlassContainer`, `GlassPanel.native`, native collector tests in `widgets/src/glass_panel.rs` | Landed. |
| aichat wiring | `examples/aichat/src/main.rs` uses one `GlassContainer` and four native panels | Landed for AppleNativeUnderlay. |
| Phase F semantic layer | `aichat-liquid-glass-phase-f-audit.md`; `GlassButton`, variants, separators, toolbar, shell/sidebar/main/composer/card surfaces | Landed for Makepad-rendered controls. |
| Native interactive controls | `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; `APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`; spec states native controls remain separate from panel descriptors | Research and policy only; not implemented. |
| UIKit backend | `aichat-liquid-glass-step-44-ios-native-glass-unsupported.spec`; iOS emits explicit unsupported fallback | UIKit backend is not implemented. |
| UIKit SDK gate | `aichat-liquid-glass-step-45-phase-g-sdk-evidence.spec`; local `iPhoneOS18.5` headers have no `UIGlassEffect` / `UIGlassContainerEffect` | Blocked until iOS 26 SDK/runtime validation. |
| Full native interior glass | `aichat-liquid-glass-v4.2-route-decision.md`; underlay cannot produce recognizable interior treatment | full native interior Liquid Glass is not implemented. |
| AppleNativeInterleave backend name | `aichat-liquid-glass-step-48-native-interleave-backend-guard.spec`; `AICHAT_GLASS_BACKEND=apple-native-interleave` falls back to shader with a renderer-split warning | Guarded as reserved, not implemented. |
| AppleNativeInterleave renderer requirements | `APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md` | Requirements recorded; no prototype implementation yet. |
| ShaderBackdrop fallback route | `aichat-liquid-glass-v4.2-route-decision.md` selects `ShaderBackdropInterior` for complete aichat interior visuals | Separate route, not full Apple-native support. |
| Phase H advanced behavior | `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`; `APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`; Step 53 adds explicit fullscreen native fallback/suppression; Step 54 adds `native-display-change` backing-scale probe; Step 55 adds weaker native inactive-window app-side dimming; Step 56 adds `native-container-spacing` probe; Step 58 adds `GlassScrollEdge` widgets; Step 59 adds staged aichat wiring; Step 60 drives edge visibility from `PortalList` state; Step 61 ramps top edge opacity from scroll offset; Step 62 exposes bottom scroll distance and ramps bottom edge opacity; Step 63 adds a native spacing animation probe runnable; Step 64 adds a native inactive-window probe runnable | Phase H is gate-defined/probed only; fullscreen has explicit fallback; scroll edge visibility and strength are state-driven; animated spacing has a Studio probe but still needs visual drift validation; native inactive has a log probe but still needs visual focus-transition validation; multi-display remains unproven. |
| Phase I popup/modal glass | `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`; v4 spec lists separate `NSPanel` / `UIWindow` popup/modal glass design | Phase I is gate-defined only; not implemented. |
| Release limitations | `aichat-liquid-glass-v4.1-release-notes.md` lists runtime switching, fullscreen, Stage Manager, multiple displays, rounded-corner sync, popup/modal, iOS backend | Documented limitations, not solved. |

## Validation Evidence

Recent verified gates:

- `cargo check -p makepad-widgets`
- `cargo check -p makepad-example-aichat`
- `cargo check -p makepad-platform`
- `agent-spec parse/lint` for Steps 43-46
- `rustfmt --check` and `git diff --check`
- Studio release run `makepad-example-aichat-apple-native-underlay-clear`
  produced `state=4`, `style=clear`, `containers=1`,
  `panels_installed=4`, `panels_failed=0`.

These gates prove the current macOS AppleNativeUnderlay path and semantic
widget layer. They do not prove UIKit support, native interactive controls,
advanced native behavior, or full native interior Liquid Glass.

## Incomplete Requirements

- UIKit backend is not implemented.
- Native interactive controls are not implemented.
- full native interior Liquid Glass is not implemented.
- Phase H is not implemented.
- Phase I is not implemented.
- Runtime switching remains a known limitation.
- Fullscreen-specific native glass behavior remains a known limitation.
- Stage Manager and multi-display behavior remain known limitations.
- Popup/modal native glass remains a separate phase.

## Next Gates

1. Install or select an iOS 26 SDK/runtime, then replace the explicit iOS
   unsupported fallback with a UIKit prototype that creates one
   `UIGlassContainerEffect` and multiple `UIGlassEffect` panels.
2. If full Apple-native interior glass remains a requirement, build an
   `AppleNativeInterleave` prototype with lower Metal content, native glass,
   and upper Makepad foreground content. The current underlay path is not
   sufficient.
3. Before enabling native interactive controls, define event ownership and
   hit-test forwarding for AppKit/UIKit controls.
4. Scope Phase H and Phase I separately so fullscreen/multi-display/popup
   behavior does not destabilize the landed macOS underlay backend. The
   concrete gates are tracked in `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`.
