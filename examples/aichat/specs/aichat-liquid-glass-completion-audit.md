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
| Native interactive controls | `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; `APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`; Step 100 closes the first hit-test ownership decision: explicit native controls use direct AppKit/UIKit hit testing, actions bridge back into Makepad, and ordinary glass panels stay passthrough; Step 101 adds shared native control descriptors and v4.10 validation; Step 102 adds the platform op transport; Step 103 extends GlassContainer collector plumbing for future control descriptors; Step 104 adds opt-in Button descriptor export while keeping native installation absent; Step 105 adds `NativeGlassControlActivatedEvent` and maps matching native button activations back to `ButtonAction::Clicked`; Step 106 adds `NativeGlassControlDescriptor.label` and exports `Button.text`; Step 107 adds macOS validation/logging for control batches; Step 108 adds a macOS `NSButton` installer skeleton and `NativeGlassControlTarget` action bridge; Step 109 adds an opt-in aichat probe for `clear_button` and `send_button`; Step 110 adds Studio runnable `makepad-example-aichat-macos-native-clear-control-probe`; Step 111 records Studio build `[111]` runtime logs showing `controls_total=2 controls_visible=2` and `installed-appkit-buttons`; Step 112 adds `target-action` and `button-action` diagnostics for real click validation; Step 113 inserts macOS native controls at the top of the AppKit container; Step 114 adds probe-only `makepad-click` diagnostics for aichat buttons; Step 115 records a system-click attempt against build `[114]` with no activation logs; Step 116 replaces raw `NSButton` allocation with a `NativeGlassButton` subclass that logs first-mouse, hit-test, and mouse-down diagnostics; Step 117 adds probe-gated Metal view hit-test and mouse-down diagnostics; Step 118 records build `[116]` click attempts and `QueryLogs` evidence showing no post-click diagnostics; Step 119 adds AppKit container hierarchy dump logs after native control installation and exposes a native-control geometry mismatch | Research, hit-test ownership decision, shared descriptor model, platform transport, widget collector plumbing, opt-in Button export, shared activation bridge, label payload, macOS control diagnostics, macOS `NSButton` installer skeleton, aichat probe gate, Studio control-probe runnable, first macOS installer runtime log, activation diagnostics, topmost AppKit insertion, Makepad click diagnostics, failed/inconclusive system-click attempts, native button hit-test diagnostics, Metal view hit-test diagnostics, post-click log-query evidence, and hierarchy diagnostics landed; native interactive controls are not complete because native control geometry does not yet align with Makepad widgets, real AppKit click delivery is unproven, glass-specific button styling is unproven, UIKit controls are missing, and accessibility remains unresolved. |
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
- Studio release run
  `makepad-example-aichat-macos-native-clear-control-probe` produced build
  `[111]`, `native-control-probe=buttons-enabled`,
  `backend=apple-native-controls state=Installed
  reason=installed-appkit-buttons controls_total=2 controls_visible=2`, and
  native underlay `panels_installed=4 panels_failed=0`.
- Step 112 adds `event=target-action` and `event=button-action` diagnostics so
  the next real AppKit click validation has direct log evidence.
- Step 113 inserts native macOS control views at the top of the AppKit
  container; it still needs a real click run to prove hit testing.
- Step 114 adds `native-control-probe=makepad-click` logs to distinguish
  Makepad-delivered clicks from AppKit `target-action` delivery.
- Step 115 records a build `[114]` system-click attempt at the mapped
  clear-button coordinate. It produced no `target-action`, `button-action`, or
  `makepad-click` logs, so it is not a passing native-control validation.
- Step 116 adds a `NativeGlassButton` subclass with `accepts-first-mouse`,
  `button-hit-test`, and `button-mouse-down` diagnostics for the next runtime
  click attempt.
- Step 117 adds probe-gated `metal-view-hit-test` and `metal-view-mouse-down`
  diagnostics to tell whether the system click is reaching the Makepad view
  instead of the native button.
- Step 118 records build `[116]` click attempts plus `QueryLogs` since index
  `8154`; no native button, Metal view, target/action, or Makepad click
  diagnostics appeared after the clicks.
- Step 119 adds AppKit hierarchy logs after native control installation. These
  can prove view order and frame placement, but not real AppKit click delivery.
- Step 119 runtime evidence shows `clear_button`'s native frame does not match
  the Studio `WidgetQuery` rect. Two attempted geometry fixes were rejected:
  `clipped_rect(cx)` produced `empty-visible-control-rect`, and a view-origin
  transform produced invalid offscreen frames.
- Step 121 adds `native-control-frame` logs with labels and records that Studio
  `WidgetQuery` / `Click` coordinates are not directly comparable to AppKit
  subview frames. Studio screenshots capture the Metal framebuffer and do not
  prove native AppKit sibling-control visual alignment.
- Step 122 adds an AppKit-side hit-test probe after native control insertion.
  Build `[127]` produced `event=appkit-hit-test-probe ... result_class=NativeGlassButton
  matches_control=true` for both `Clear` and `↑`, proving AppKit hit testing
  resolves the installed native controls at their center points.
- Step 123 adds
  `makepad-example-aichat-macos-native-clear-control-action-probe`, gated by
  `MAKEPAD_NATIVE_GLASS_CONTROL_PERFORM_CLICK_PROBE=Clear`. Build `[130]`
  produced `perform-click-probe -> target-action -> button-action ->
  native-control-probe=makepad-click id=clear_button`, proving the AppKit
  target/action bridge reaches Makepad's existing button action path.
- Step 124 adds
  `makepad-example-aichat-macos-native-clear-control-mouse-probe`, gated by
  `MAKEPAD_NATIVE_GLASS_CONTROL_MOUSE_EVENT_PROBE=Clear`. Build `[133]`
  produced `synthetic-mouse-probe -> button-mouse-down -> target-action ->
  button-action -> native-control-probe=makepad-click id=clear_button`,
  proving an AppKit mouse event posted through the app event queue reaches the
  native button and re-enters Makepad.
- Step 125 adds
  `makepad-example-aichat-macos-native-clear-control-cgevent-probe`, gated by
  `MAKEPAD_NATIVE_GLASS_CONTROL_CGEVENT_PROBE=Clear`. Builds `[135]` and `[136]`
  produced `cg-event-probe` logs, but no subsequent `button-mouse-down`,
  `target-action`, `button-action`, or `native-control-probe=makepad-click`
  logs. The system-click gate remains open.
- Step 126 adds
  `makepad-example-aichat-macos-native-clear-control-cgevent-pid-probe`, gated
  by `MAKEPAD_NATIVE_GLASS_CONTROL_CGEVENT_PROBE=pid:Clear`. Build `[138]`
  produced `cg-event-probe mode=Process`, but no subsequent
  `button-mouse-down`, `target-action`, `button-action`, or
  `native-control-probe=makepad-click` logs. PID-targeted CGEvent delivery did
  not close the system-click gate.
- Step 127 makes iOS `SetNativeGlassControlBatch` observable instead of silent:
  valid batches log
  `backend=apple-native-ios-controls state=Unsupported reason=installer-not-implemented`;
  invalid batches log stable rejection reasons. This does not implement UIKit
  native controls.
- Step 128 runs
  `makepad-example-aichat-macos-native-clear-fullscreen-probe` in Studio release
  build `[139]`. The app reached State 4 and requested fullscreen, but logged
  `native-fullscreen-probe=timeout phase=enter` with no later fallback, restore,
  exit, or native panel frame evidence. Fullscreen remains unproven.
- Step 129 runs
  `makepad-example-aichat-macos-native-clear-spacing-probe` in Studio release
  build `[140]`. The probe ran frame `0 -> 120`, spacing `12 -> 36 -> 12`, and
  repeatedly logged `native-container-spacing` while the native batch remained
  Installed. Morphing visual quality and resize-after-animation remain
  unproven.
- Step 130 runs
  `makepad-example-aichat-macos-native-clear-inactive-probe` in Studio release
  build `[141]`. It logged `native-inactive-probe active=true style=clear
  multiplier=1.000`, but activating Finder did not produce `active=false`
  evidence. Inactive-window behavior remains unproven.
- Step 131 runs
  `makepad-example-aichat-macos-native-clear-geometry-probe` in Studio release
  build `[142]`. The app reached State 4, but System Events reported no
  scriptable `makepad-example-aichat` window, so no geometry-change frame
  snapshot evidence was produced. Stage Manager/split-view and resize/reposition
  behavior remain unproven.
- Step 132 makes the geometry probe self-driven after
  `installed-native-glass-batch` and makes macOS programmatic resize/reposition
  ops emit `WindowGeomChange` with explicit old geometry. Studio builds `[146]`
  and `[147]` logged `native-geometry-probe=request-resize`, but still produced
  no `native-display-frame-snapshot reason=geometry-change` evidence.
- Step 133 fixes the reentrant callback problem in the programmatic geometry
  path. Studio release build `[150]` logged
  `native-geometry-op=resize changed=true`,
  `native-geometry-op=reposition changed=true`, and two
  `native-display-frame-snapshot reason=geometry-change` entries, each followed
  by four `native-panel-frame` entries.
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
- Native interactive controls are only partially implemented. Steps 100-112
  cover the first hit-test ownership decision, shared descriptors, platform op
  transport, collector plumbing, opt-in Button export, shared activation event
  mapping, labels, macOS validation, an AppKit `NSButton` installer skeleton,
  the aichat control probe, the Studio runnable, installer runtime logs, and
  target/action diagnostics, topmost AppKit insertion, and Makepad click
  diagnostics. They are still incomplete because macOS 26 glass-specific button
  styling is unproven, `makepad-click` logs alone do not prove native delivery,
  a real AppKit click still needs to produce `target-action` and
  `button-action` logs, the Step 115 system-click attempt produced no activation
  logs, Step 118 shows the automated build `[116]` click attempts produced no
  post-click diagnostics, Step 121 shows Studio framebuffer/query evidence is
  insufficient for native AppKit sibling-control visual validation, Step 122
  proves AppKit center-point hit testing, Step 123 proves programmatic AppKit
  target/action bridging, and Step 124 proves synthetic AppKit mouse events
  posted through the application queue. Step 125 retains a CGEvent system-click
  probe, and Step 126 adds a PID-targeted `CGEventPostToPid` variant, but
  current builds `[135]`, `[136]`, and `[138]` did not produce native click
  delivery. A physical user/system click is still unproven. UIKit controls are
  not installed; Step 127 only validates/logs unsupported iOS control batches.
  Accessibility ownership is
  unresolved, and non-macOS backends remain no-op.
- full native interior Liquid Glass is not implemented; Step 96 proves lower
  scene pass routing, and Step 98 records that the current native interleave
  visual result has no transparency/refraction/liquid distortion.
- Phase H is not implemented.
- Step 128 confirms the fullscreen probe path is reachable, but fullscreen
  enter/exit validation did not complete in Studio build `[139]`.
- Step 129 confirms animated spacing updates keep the macOS native batch
  installed, but it does not prove final morphing visual quality.
- Step 130 confirms native inactive logging for the active state only; inactive
  transition behavior is still unproven.
- Step 133 closes the self-driven same-screen geometry snapshot gap, but real
  Stage Manager, split-view, and multi-display geometry behavior remains
  unproven.
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
