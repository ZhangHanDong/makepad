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
| macOS AppKit backend | `platform/src/os/apple/macos/macos_window.rs`; Studio logs `state=4 substrate=macos-native style=clear style_raw=1`; `containers=1 panels_installed=4 panels_failed=0`; macOS system screenshot `/Users/zhangalex/Desktop/截屏2026-05-09 18.15.46.png` shows broad user-visible transparency/blur/refraction over desktop wallpaper for `macos-native-clear` | Landed for AppleNativeUnderlay. |
| Shared descriptors | `NativeGlassBatch`, `NativeGlassContainerDescriptor`, `NativeGlassPanelDescriptor` in `platform/src/event/window.rs` | Landed. |
| Descriptor validation | `validate_v4_1`, max 12 panels, single container, reject `NativeGlassHitTest::Interactive` | Landed for v4.1 constraints. |
| Makepad widget export | `GlassContainer`, `GlassPanel.native`, native collector tests in `widgets/src/glass_panel.rs` | Landed. |
| aichat wiring | `examples/aichat/src/main.rs` uses one `GlassContainer` and four native panels | Landed for AppleNativeUnderlay. |
| aichat iOS native event handling | Step 76 adds iOS substrate style variants; Step 77 maps installed `IosGlassRegular` / `IosGlassClear` events to `GlassSubstrate::IosNative` so aichat uses native overlay tuning after UIKit backend success | Event handling landed; iOS runtime validation still missing. |
| Phase F semantic layer | `aichat-liquid-glass-phase-f-audit.md`; `GlassButton`, variants, separators, toolbar, shell/sidebar/main/composer/card surfaces | Landed for Makepad-rendered controls. |
| Native interactive controls | `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; `APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`; Step 100 closes the first hit-test ownership decision: explicit native controls use direct AppKit/UIKit hit testing, actions bridge back into Makepad, and ordinary glass panels stay passthrough; Step 101 adds shared native control descriptors and v4.10 validation without a platform installer | Research, hit-test ownership decision, and shared descriptor model landed; native interactive controls are not implemented. |
| UIKit backend | `aichat-liquid-glass-step-44-ios-native-glass-unsupported.spec`; Step 68 adds Objective-C runtime class preflight for `UIVisualEffectView`, `UIGlassContainerEffect`, and `UIGlassEffect`; Step 69 prepares a UIKit underlay host view with `MTKView` as a child; Step 70 adds selector preflight; Step 71 adds a dynamic installer skeleton that creates a container `UIVisualEffectView` below `MTKView` and passthrough panel `UIVisualEffectView`s when preflight passes; Step 72 adds iOS raw style overrides; Step 73 expands selector preflight to installer UIView/CALayer selectors; Step 74 adds shared batch equivalence reused by iOS; Step 76 reports iOS regular/clear through `WindowNativeSubstrateResolvedEvent` style variants; Step 77 maps those events in aichat | Installer skeleton landed; not runtime-validated on iOS 26. |
| UIKit SDK/runtime gate | `aichat-liquid-glass-step-45-phase-g-sdk-evidence.spec`; local `iPhoneOS18.5` headers have no typed `UIGlassEffect` / `UIGlassContainerEffect`; Step 68/70/73 can distinguish missing runtime classes/selectors; Step 72 allows raw style override if `regular=0` / `clear=1` is wrong | Blocked until iOS 26 runtime validation proves class availability, selector names, style raw values, visual output, rotation, safe area, keyboard, split view, and Stage Manager behavior. |
| Full native interior glass | `aichat-liquid-glass-v4.2-route-decision.md`; `aichat-liquid-glass-step-96-aichat-lower-scene-pass-probe.spec`; `aichat-liquid-glass-step-98-native-interleave-fail-verdict.spec`; Step 96 proves a dedicated aichat `DrawPassSurfaceRole::LowerScene` pass is rendered and routed to the lower scene surface, but Step 98 records the user-visible result as gray/grid only, with no transparency or recognizable blur/refraction/liquid distortion | Not complete: AppleNativeInterleave is rejected as the current production route and remains a prototype; full native interior Liquid Glass is not implemented. |
| AppleNativeInterleave backend name | `aichat-liquid-glass-step-48-native-interleave-backend-guard.spec`; `aichat-liquid-glass-step-99-interleave-fallback-to-shader-backdrop.spec`; `AICHAT_GLASS_BACKEND=apple-native-interleave` remains a known guarded value and now falls back to `ShaderBackdropInterior` with the Step 98 fail-verdict warning | Guarded as reserved, not implemented; requests are routed to the current complete interior fallback instead of plain shader. |
| AppleNativeInterleave renderer requirements | `APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`; Step 96 lower scene pass probe; Step 98 fail verdict | Requirements and a prototype probe exist; current visual verdict rejects this as production full native glass. |
| ShaderBackdrop fallback route | `aichat-liquid-glass-v4.2-route-decision.md` selects `ShaderBackdropInterior` for complete aichat interior visuals; Step 98 keeps complete interior work on ShaderBackdrop/hybrid; Step 99 makes guarded `apple-native-interleave` requests resolve to `ShaderBackdropInterior` | Separate route, not full Apple-native support. |
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
- macOS system screenshot
  `/Users/zhangalex/Desktop/截屏2026-05-09 18.15.46.png` captured the final
  AppKit/window-server composition for `makepad-example-aichat-macos-native-clear`;
  it shows desktop wallpaper and icons visible through the window with broad
  blur/refraction, which Studio framebuffer screenshots do not fully capture.

These gates prove the current macOS AppleNativeUnderlay path, semantic widget
layer, and the Step 96 lower scene pass routing prototype. The Step 98 manual
verdict rejects that prototype as production full native glass. Step 99 keeps
the reserved backend name but routes it to `ShaderBackdropInterior`. These gates
do not prove UIKit support, native interactive controls, advanced native
behavior, or full native interior Liquid Glass.

## Incomplete Requirements

- UIKit backend is not runtime-validated beyond the dynamic installer skeleton.
- Native interactive controls are not implemented; Step 100 closes the first
  hit-test ownership decision and Step 101 adds shared native control
  descriptors, but no AppKit/UIKit controls are created.
- full native interior Liquid Glass is not implemented; Step 96 proves lower
  scene pass routing, and Step 98 records that the current native interleave
  visual result has no transparency/refraction/liquid distortion.
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
2. Keep complete aichat interior glass on ShaderBackdrop or a hybrid route. A
   later AppleNativeInterleave attempt must first produce a new visual verdict
   with recognizable native transparency/refraction/liquid distortion before it
   can replace the current route.
3. Before enabling native interactive controls, define event ownership and
   hit-test forwarding for AppKit/UIKit controls.
4. Scope Phase H and Phase I separately so fullscreen/multi-display/popup
   behavior does not destabilize the landed macOS underlay backend. The
   concrete gates are tracked in `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`.
