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
| aichat iOS native event handling | Step 76 adds iOS substrate style variants; Step 77 maps installed `IosGlassRegular` / `IosGlassClear` events to `GlassSubstrate::IosNative` so aichat uses native overlay tuning after UIKit backend success | Event handling landed; iOS runtime validation still missing. |
| Phase F semantic layer | `aichat-liquid-glass-phase-f-audit.md`; `GlassButton`, variants, separators, toolbar, shell/sidebar/main/composer/card surfaces | Landed for Makepad-rendered controls. |
| Native interactive controls | `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; `APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`; spec states native controls remain separate from panel descriptors | Research and policy only; not implemented. |
| UIKit backend | `aichat-liquid-glass-step-44-ios-native-glass-unsupported.spec`; Step 68 adds Objective-C runtime class preflight for `UIVisualEffectView`, `UIGlassContainerEffect`, and `UIGlassEffect`; Step 69 prepares a UIKit underlay host view with `MTKView` as a child; Step 70 adds selector preflight; Step 71 adds a dynamic installer skeleton that creates a container `UIVisualEffectView` below `MTKView` and passthrough panel `UIVisualEffectView`s when preflight passes; Step 72 adds iOS raw style overrides; Step 73 expands selector preflight to installer UIView/CALayer selectors; Step 74 adds shared batch equivalence reused by iOS; Step 76 reports iOS regular/clear through `WindowNativeSubstrateResolvedEvent` style variants; Step 77 maps those events in aichat | Installer skeleton landed; not runtime-validated on iOS 26. |
| UIKit SDK/runtime gate | `aichat-liquid-glass-step-45-phase-g-sdk-evidence.spec`; local `iPhoneOS18.5` headers have no typed `UIGlassEffect` / `UIGlassContainerEffect`; Step 68/70/73 can distinguish missing runtime classes/selectors; Step 72 allows raw style override if `regular=0` / `clear=1` is wrong | Blocked until iOS 26 runtime validation proves class availability, selector names, style raw values, visual output, rotation, safe area, keyboard, split view, and Stage Manager behavior. |
| Full native interior glass | `aichat-liquid-glass-v4.2-route-decision.md`; `aichat-liquid-glass-step-96-aichat-lower-scene-pass-probe.spec`; Step 96 proves a dedicated aichat `DrawPassSurfaceRole::LowerScene` pass is rendered and routed to the lower scene surface, but Studio screenshots only capture that lower scene framebuffer and do not prove final native overlay sampling or recognizable Liquid Glass refraction/blur | Not complete: AppleNativeInterleave is now partially proved as a prototype route, but full native interior Liquid Glass is not implemented. |
| AppleNativeInterleave backend name | `aichat-liquid-glass-step-48-native-interleave-backend-guard.spec`; `AICHAT_GLASS_BACKEND=apple-native-interleave` falls back to shader with a renderer-split warning | Guarded as reserved, not implemented. |
| AppleNativeInterleave renderer requirements | `APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md` | Requirements recorded; no prototype implementation yet. |
| ShaderBackdrop fallback route | `aichat-liquid-glass-v4.2-route-decision.md` selects `ShaderBackdropInterior` for complete aichat interior visuals | Separate route, not full Apple-native support. |
| Phase H advanced behavior | `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`; `APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`; Step 53 adds explicit fullscreen native fallback/suppression; Step 54 adds `native-display-change` backing-scale probe; Step 55 adds weaker native inactive-window app-side dimming; Step 56 adds `native-container-spacing` probe; Step 58 adds `GlassScrollEdge` widgets; Step 59 adds staged aichat wiring; Step 60 drives edge visibility from `PortalList` state; Step 61 ramps top edge opacity from scroll offset; Step 62 exposes bottom scroll distance and ramps bottom edge opacity; Step 63 adds a native spacing animation probe runnable; Step 64 adds a native inactive-window probe runnable; Step 65 adds a native fullscreen fallback probe runnable; Step 66 adds native display frame snapshot logs; Step 67 adds a Stage Manager/split-view geometry snapshot runnable | Phase H is gate-defined/probed only; fullscreen has explicit fallback and a Studio probe but is not full native fullscreen support; scroll edge visibility and strength are state-driven; animated spacing has a Studio probe but still needs visual drift validation; native inactive has a log probe but still needs visual focus-transition validation; multi-display has frame snapshot logs but remains unproven until real display-move validation; Stage Manager/split-view has a geometry snapshot probe but remains unproven until real runtime smoke validation. |
| Phase I popup/modal glass | `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`; v4 spec lists separate `NSPanel` / `UIWindow` popup/modal glass design | Phase I is gate-defined only; not implemented. |
| Release limitations | `aichat-liquid-glass-v4.1-release-notes.md` lists runtime switching, fullscreen, Stage Manager, multiple displays, rounded-corner sync, popup/modal, iOS backend | Documented limitations, not solved. |

## Validation Evidence

Recent verified gates:

- `cargo check -p makepad-widgets`
- `cargo check -p makepad-example-aichat`
- `cargo check -p makepad-platform`
- `cargo check -p makepad-platform --target aarch64-apple-ios --release`
- Step 71-77 focused release unit tests for iOS selector/style state,
  descriptor batch equivalence, and aichat iOS native event handling
- `agent-spec parse/lint` for Steps 43-46
- `rustfmt --check` and `git diff --check`
- Studio release run `makepad-example-aichat-apple-native-underlay-clear`
  produced `state=4`, `style=clear`, `containers=1`,
  `panels_installed=4`, `panels_failed=0`.

These gates prove the current macOS AppleNativeUnderlay path, semantic widget
layer, and the Step 96 lower scene pass routing prototype. They do not prove
UIKit support, native interactive controls, advanced native behavior, or full
native interior Liquid Glass.

## Incomplete Requirements

- UIKit backend is not runtime-validated beyond the dynamic installer skeleton.
- Native interactive controls are not implemented.
- full native interior Liquid Glass is not implemented; Step 96 proves lower
  scene pass routing, but native glass visual sampling over that lower scene
  still needs manual visual validation and production integration.
- Phase H is not implemented.
- Phase I is not implemented.
- Runtime switching remains a known limitation.
- Fullscreen-specific native glass behavior remains a known limitation.
- Stage Manager and multi-display behavior remain known limitations.
- Popup/modal native glass remains a separate phase.

## Next Gates

1. Run iOS 26 runtime validation against the dynamic UIKit installer skeleton:
   prove class availability, selector names, style raw values, visual output,
   rotation, safe area, keyboard, split view, and Stage Manager behavior. If
   `regular=0` / `clear=1` is wrong, use `AICHAT_IOS_GLASS_STYLE_REGULAR_RAW`
   and `AICHAT_IOS_GLASS_STYLE_CLEAR_RAW` to validate corrected values before
   changing defaults.
2. If full Apple-native interior glass remains a requirement, visually validate
   the Step 96 `AppleNativeInterleave` prototype: prove that native glass
   visibly samples the aichat lower scene pass with recognizable blur/refraction,
   then split production aichat rendering into lower scene and upper UI passes.
   The current underlay path is not sufficient.
3. Before enabling native interactive controls, define event ownership and
   hit-test forwarding for AppKit/UIKit controls.
4. Scope Phase H and Phase I separately so fullscreen/multi-display/popup
   behavior does not destabilize the landed macOS underlay backend. The
   concrete gates are tracked in `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`.
