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
| Native interactive controls | `APPLE-NATIVE-BUTTON-GLASS-RESEARCH.md`; `APPLE-NATIVE-INTERACTIVE-CONTROLS-POLICY.md`; Step 100 closes the first hit-test ownership decision: explicit native controls use direct AppKit/UIKit hit testing, actions bridge back into Makepad, and ordinary glass panels stay passthrough; Step 101 adds shared native control descriptors and v4.10 validation; Step 102 adds the platform op transport; Step 103 extends GlassContainer collector plumbing for future control descriptors; Step 104 adds opt-in Button descriptor export while keeping native installation absent; Step 105 adds `NativeGlassControlActivatedEvent` and maps matching native button activations back to `ButtonAction::Clicked`; Step 106 adds `NativeGlassControlDescriptor.label` and exports `Button.text`; Step 107 adds macOS validation/logging for control batches; Step 108 adds a macOS `NSButton` installer skeleton and `NativeGlassControlTarget` action bridge; Step 109 adds an opt-in aichat probe for `clear_button` and `send_button`; Step 110 adds Studio runnable `makepad-example-aichat-macos-native-clear-control-probe`; Step 111 records Studio build `[111]` runtime logs showing `controls_total=2 controls_visible=2` and `installed-appkit-buttons`; Step 112 adds `target-action` and `button-action` diagnostics for real click validation; Step 113 inserts macOS native controls at the top of the AppKit container; Step 114 adds probe-only `makepad-click` diagnostics for aichat buttons; Step 115 records a system-click attempt against build `[114]` with no activation logs; Step 116 replaces raw `NSButton` allocation with a `NativeGlassButton` subclass that logs first-mouse, hit-test, and mouse-down diagnostics; Step 117 adds probe-gated Metal view hit-test and mouse-down diagnostics; Step 118 records build `[116]` click attempts and `QueryLogs` evidence showing no post-click diagnostics; Step 119 adds AppKit container hierarchy dump logs after native control installation and exposed an early native-control geometry mismatch; Step 121 clarifies Studio query/framebuffer evidence limits and adds labeled native-control frame logs; Step 122 proves AppKit center-point hit testing resolves the installed native controls; Step 123 proves programmatic AppKit target/action bridging into Makepad; Step 124 proves synthetic AppKit mouse events posted through the app queue re-enter Makepad; Steps 125-126 retain failing CGEvent system-click probes; Step 144 configures installed macOS native buttons with `NSButton.BezelStyle.glass` raw value `16` and an override env; Step 145 fixes the retained CGEvent probe's AppKit-screen to Quartz-event coordinate conversion; Step 146 mirrors descriptor labels into macOS native button accessibility labels; Step 149 proves accessibility activation reaches target/action/Makepad click; Step 150 adds window-level mouse event diagnostics for the next physical click run; Step 151 reruns the retained CGEvent probe with Step 150 diagnostics and shows it still does not enter `RenderWindow.sendEvent:` | Research, hit-test ownership decision, shared descriptor model, platform transport, widget collector plumbing, opt-in Button export, shared activation bridge, label payload, macOS control diagnostics, macOS `NSButton` installer skeleton, aichat probe gate, Studio control-probe runnable, first macOS installer runtime log, activation diagnostics, topmost AppKit insertion, Makepad click diagnostics, failed/inconclusive system-click attempts, native button hit-test diagnostics, Metal view hit-test diagnostics, post-click log-query evidence, hierarchy diagnostics, AppKit hit-test proof, programmatic target/action proof, synthetic AppKit mouse-event proof, macOS glass bezel configuration, CGEvent coordinate conversion, macOS accessibility label mirroring, macOS accessibility activation proof, window-level physical-click diagnostics, and CGEvent/window diagnostic rerun landed; native interactive controls are not complete because physical/system AppKit click delivery is still unproven until a physical click produces `window-send-event`, `button-mouse-down`, `target-action`, `button-action`, and `native-control-probe=makepad-click`, and iOS controls still lack real iOS 26 runtime validation. |
| UIKit backend | `aichat-liquid-glass-step-44-ios-native-glass-unsupported.spec`; Step 68 adds Objective-C runtime class preflight for `UIVisualEffectView`, `UIGlassContainerEffect`, and `UIGlassEffect`; Step 69 prepares a UIKit underlay host view with `MTKView` as a child; Step 70 adds selector preflight; Step 71 adds a dynamic installer skeleton that creates a container `UIVisualEffectView` below `MTKView` and passthrough panel `UIVisualEffectView`s when preflight passes; Step 72 adds iOS raw style overrides; Step 73 expands selector preflight to installer UIView/CALayer selectors; Step 74 adds shared batch equivalence reused by iOS; Step 76 reports iOS regular/clear through `WindowNativeSubstrateResolvedEvent` style variants; Step 77 maps those events in aichat | Installer skeleton landed; not runtime-validated on iOS 26. |
| UIKit SDK/runtime gate | `aichat-liquid-glass-step-45-phase-g-sdk-evidence.spec`; local `iPhoneOS18.5` headers have no typed `UIGlassEffect` / `UIGlassContainerEffect`; Step 68/70/73 can distinguish missing runtime classes/selectors; Step 72 allows raw style override if `regular=0` / `clear=1` is wrong | Blocked until iOS 26 runtime validation proves class availability, selector names, style raw values, visual output, rotation, safe area, keyboard, split view, and Stage Manager behavior. |
| Full native interior glass | `aichat-liquid-glass-v4.2-route-decision.md`; `aichat-liquid-glass-step-96-aichat-lower-scene-pass-probe.spec`; `aichat-liquid-glass-step-98-native-interleave-fail-verdict.spec`; Step 96 proves a dedicated aichat `DrawPassSurfaceRole::LowerScene` pass is rendered and routed to the lower scene surface, but Step 98 records the user-visible result as gray/grid only, with no transparency or recognizable blur/refraction/liquid distortion | Not complete: AppleNativeInterleave is rejected as the current production route and remains a prototype; full native interior Liquid Glass is not implemented. |
| AppleNativeInterleave backend name | `aichat-liquid-glass-step-48-native-interleave-backend-guard.spec`; `aichat-liquid-glass-step-99-interleave-fallback-to-shader-backdrop.spec`; `AICHAT_GLASS_BACKEND=apple-native-interleave` remains a known guarded value and now falls back to `ShaderBackdropInterior` with the Step 98 fail-verdict warning | Guarded as reserved, not implemented; requests are routed to the current complete interior fallback instead of plain shader. |
| AppleNativeInterleave renderer requirements | `APPLE-NATIVE-INTERLEAVE-RENDERER-REQUIREMENTS.md`; Step 96 lower scene pass probe; Step 98 fail verdict | Requirements and a prototype probe exist; current visual verdict rejects this as production full native glass. |
| ShaderBackdrop fallback route | `aichat-liquid-glass-v4.2-route-decision.md` selects `ShaderBackdropInterior` for complete aichat interior visuals; Step 98 keeps complete interior work on ShaderBackdrop/hybrid; Step 99 makes guarded `apple-native-interleave` requests resolve to `ShaderBackdropInterior` | Separate route, not full Apple-native support. |
| Phase H advanced behavior | `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`; `APPLE-NATIVE-SCROLL-EDGE-GLASS-POLICY.md`; Step 53 adds explicit fullscreen native fallback/suppression; Step 54 adds `native-display-change` backing-scale probe; Step 55 adds weaker native inactive-window app-side dimming; Step 56 adds `native-container-spacing` probe; Step 58 adds `GlassScrollEdge` widgets; Step 59 adds staged aichat wiring; Step 60 drives edge visibility from `PortalList` state; Step 61 ramps top edge opacity from scroll offset; Step 62 exposes bottom scroll distance and ramps bottom edge opacity; Step 63 adds a native spacing animation probe runnable; Step 64 adds a native inactive-window probe runnable; Step 65 adds a native fullscreen fallback probe runnable; Step 66 adds native display frame snapshot logs; Step 67 adds a Stage Manager/split-view geometry snapshot runnable | Phase H is gate-defined/probed only; fullscreen has explicit fallback and a Studio probe but is not full native fullscreen support; scroll edge visibility and strength are state-driven; animated spacing has a Studio probe but still needs visual drift validation; native inactive has a log probe but still needs visual focus-transition validation; multi-display has frame snapshot logs but remains unproven until real display-move validation; Stage Manager/split-view has a geometry snapshot probe but remains unproven until real runtime smoke validation. |
| Phase I popup/modal glass | `APPLE-NATIVE-ADVANCED-BEHAVIOR-GATES.md`; `APPLE-NATIVE-POPUP-MODAL-GLASS-POLICY.md`; v4 spec lists separate `NSPanel` / `UIWindow` popup/modal glass design; Step 136 adds the gated `MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE` macOS popup installer and Studio runnables `makepad-example-aichat-macos-native-clear-transient-probe` / `makepad-example-aichat-macos-native-clear-transient-dismiss-probe`; Studio release build `[161]` logs `transient-window=popup state=Installed substrate=macos-native style=clear` and `transient-window-probe=draw popup_size=(240.0,160.0)` while the main native batch remains `panels_installed=4 panels_failed=0`; Studio captured the popup framebuffer at `/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-161-kind-0-req-49-1778342316874.png`; Studio release build `[166]` logs `transient-window=popup-dismiss-probe request=dispatch-popup-dismissed` and aichat receives `transient-window-probe=dismissed popup=WindowId(1, 0) reason=FocusLost`; Step 137 Studio release build `[168]` logs `transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1` and popup-window `panels_installed=1 panels_failed=0` while the main window keeps `panels_installed=4 panels_failed=0`; Step 138 adds stable iOS popup transient unsupported logging and `cargo check -p makepad-platform --target aarch64-apple-ios --release` passes; Step 142 removes the previous iOS test-target `--no-run` blocker for `ios_native_glass_transient`; Step 139 adds macOS AppKit Escape-to-`PopupDismissed(Escape)` routing and Studio build `[169]` confirms popup native glass setup still works after the route change; Step 140 adds outside-click routing and Studio build `[171]` logs platform `event=outside-click` plus aichat `reason=OutsideClick`; Step 141 adds a default-off in-window `Modal.native_glass` unsupported gate and focused widget test | macOS transient native substrate install, popup Metal draw-pass composition, popup widget-tree descriptor export, synthetic platform-to-app `PopupDismissed` delivery, iOS transient unsupported observability, iOS test-target compilation for the transient unsupported-log tests, macOS Escape route implementation, Studio/Makepad outside-click dismissal, and in-window Modal unsupported observability are proved for probe/code paths; Phase I is still incomplete because physical AppKit outside-click/Escape evidence, UIKit transient implementation/runtime validation, and separate-platform-window modal native glass remain unproven. |
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
- Step 144 configures installed macOS native buttons with
  `NSButton.BezelStyle.glass` raw value `16`. Studio release build `[172]`
  logged `button-style ... label="Clear" bezel=glass raw=16` and
  `button-style ... label="↑" bezel=glass raw=16` while the native control
  batch remained `state=Installed reason=installed-appkit-buttons
  controls_total=2 controls_visible=2`.
- Step 145 fixes the retained CGEvent probe coordinate conversion. Studio
  release builds `[173]` and `[174]` logged the corrected
  `screen=(2004.5,1024.0) cg=(2004.5,416.0)` conversion for global and
  process-targeted CGEvent probes, but neither run produced
  `button-mouse-down`, `target-action`, `button-action`, or
  `native-control-probe=makepad-click`.
- Step 146 mirrors macOS native button descriptor labels into AppKit
  accessibility labels. Studio release build `[2]` of
  `makepad-example-aichat-macos-native-clear-control-probe` logged
  `accessibility-label ... label="Clear"` and
  `accessibility-label ... label="↑"` while the native-control batch remained
  installed with glass bezel raw value `16`.
- Step 151 reruns
  `makepad-example-aichat-macos-native-clear-control-cgevent-probe` in Studio
  release build `[3]` after Step 150 added `NSWindow.sendEvent:` diagnostics.
  The run logged State 4, `controls_total=2 controls_visible=2`, matching
  `appkit-hit-test-probe` results for both native buttons, and
  `cg-event-probe mode=Global ... screen=(2004.5,1024.0) cg=(2004.5,416.0)`.
  No `window-send-event`, `button-mouse-down`, `target-action`,
  `button-action`, or `native-control-probe=makepad-click` logs followed, so
  the retained CGEvent probe still does not close the physical/system click
  gate.
- Step 127 makes iOS `SetNativeGlassControlBatch` observable instead of silent:
  valid batches log
  `backend=apple-native-ios-controls state=Unsupported reason=installer-not-implemented`;
  invalid batches log stable rejection reasons. This does not implement UIKit
  native controls.
- Step 147 adds iOS native-control class/selector preflight for `UIButton`,
  `UIButtonConfiguration`, ordinary button selectors, and the iOS 26
  `glassButtonConfiguration` / `clearGlassButtonConfiguration` selectors. The
  iOS target `--no-run` test build and release check pass, but UIKit runtime
  installation and iOS 26 validation remain open.
- Step 148 adds a compiled UIKit native-control installer skeleton that creates
  `UIButton` mirrors, applies `UIButtonConfiguration.glass()` / `clearGlass()`,
  mirrors descriptor labels into title/accessibility label, and routes
  `touchUpInside` into `NativeGlassControlActivatedEvent`. The iOS target
  `--no-run` test build and release checks pass, but real iOS 26 visual/input
  validation remains open.
- Step 128 runs
  `makepad-example-aichat-macos-native-clear-fullscreen-probe` in Studio release
  build `[139]`. The app reached State 4 and requested fullscreen, but logged
  `native-fullscreen-probe=timeout phase=enter` with no later fallback, restore,
  exit, or native panel frame evidence. Fullscreen remains unproven.
- Step 134 reruns the fullscreen probe in Studio release build `[155]` after
  adding `NSWindowCollectionBehaviorFullScreenPrimary` for standard windows and
  routing programmatic fullscreen/normalize ops through the internal
  `WindowGeomChange` path. The run logged
  `native-fullscreen-probe=observed-enter`,
  `fullscreen-native-fallback=shader reason=fullscreen-enter`,
  `native-fullscreen-probe=request-exit`,
  `fullscreen-native-restore=apple-native-underlay reason=fullscreen-exit`, and
  `native-fullscreen-probe=observed-exit`.
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
- Step 135 adds app-level activation event handling so
  `applicationDidBecomeActive:` maps to `WindowGotFocus` and
  `applicationDidResignActive:` maps to `WindowLostFocus`. Studio release build
  `[157]` reached State 4 and logged `active=true`, but the current automation
  environment kept the host process frontmost, so Finder activation did not
  produce app resign-active or `active=false` evidence.
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
- Step 136 first runs
  `makepad-example-aichat-macos-native-clear-transient-probe` in Studio release
  build `[160]`. The app reached main-window State 4, opened a real Makepad
  popup `WindowId(1, 0)`, and the macOS platform logged
  `transient-window=popup state=Installed substrate=macos-native style=clear
  reason=installed-on-proofed-hierarchy`. The same run kept the main native
  batch installed with `panels_installed=4 panels_failed=0`. Studio automation
  did not prove popup visual composition or `PopupDismissed` delivery.
- Step 136 then adds a dedicated transient popup draw pass. Studio release
  build `[161]` logged `transient-window-probe=draw popup_size=(240.0,160.0)`
  and captured the popup framebuffer at
  `/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-161-kind-0-req-49-1778342316874.png`,
  proving the popup can present Makepad Metal content above its independent
  native glass substrate.
- Step 136 adds
  `makepad-example-aichat-macos-native-clear-transient-dismiss-probe`, gated by
  `MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE=dismiss`. Studio release build `[166]`
  logged `transient-window=popup-dismiss-probe
  request=dispatch-popup-dismissed`, and aichat logged
  `transient-window-probe=dismissed popup=WindowId(1, 0) reason=FocusLost`.
  This proves synthetic platform-to-app `PopupDismissed` delivery. Physical
  outside-click and Escape dismissal remain unproven.
- Step 137 changes the transient popup probe to draw a popup-local
  `GlassContainer` widget tree. Studio release build `[168]` logged
  `transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1`,
  then installed a popup-window native batch with
  `panels_installed=1 panels_failed=0` and `style=clear style_raw=1`. The main
  window stayed installed with `panels_installed=4 panels_failed=0`. Studio
  captured the popup framebuffer at
  `/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-168-kind-0-req-98-1778344085662.png`.
- Step 138 adds stable iOS transient popup unsupported logging. When
  `MAKEPAD_NATIVE_GLASS_TRANSIENT_PROBE` is enabled and iOS creates a popup,
  the backend logs
  `transient-window=popup state=Unsupported substrate=ios-native style=clear reason=transient-platform-window-missing`
  for Apple-native clear requests, or a stable `backend-not-native` rejection
  for non-native requests. `cargo check -p makepad-platform --target
  aarch64-apple-ios --release` passes. Step 142 removes the previous
  `live_reload.rs` test-target compile blocker, and
  `cargo test -p makepad-platform ios_native_glass_transient --target
  aarch64-apple-ios --release --no-run` now compiles the iOS test executable.
  This remains compile/observability evidence, not runtime UIKit proof.
- Step 139 adds macOS AppKit `keyDown:` Escape routing for popup windows. The
  route dispatches `PopupDismissed(reason=Escape)` and logs
  `transient-window=popup-dismiss event=escape` when it fires. Studio release
  build `[169]` revalidated the popup native glass setup after this change, but
  `osascript` and `cliclick` Escape injection attempts produced no post-input
  logs. Physical/system Escape validation remains open.
- Step 140 adds outside-click dismissal routing for the macOS mouse-down event
  path and the shared Studio input path. The focused unit test
  `popup_outside_click_helper` passes. Studio release build `[171]` logged
  `transient-window=popup-dismiss event=outside-click popup=WindowId(1, 0) source_window=WindowId(0, 0)`
  followed by
  `transient-window-probe=dismissed popup=WindowId(1, 0) reason=OutsideClick`
  after a Studio remote click at `(300, 300)`. This proves the Studio/Makepad
  shared input path; physical AppKit outside-click validation remains open.
- Step 141 adds a default-off `native_glass` opt-in to `widgets::Modal`. Since
  the current Modal is rendered inside the existing Makepad window and does not
  own a separate platform window, opening it with `native_glass: true` logs
  `transient-window=modal state=Unsupported reason=in-window-modal-has-no-platform-window`.
  `cargo test -p makepad-widgets
  modal_native_glass_unsupported_log_line_is_stable --release` passes.

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
  diagnostics. Step 144 configures installed macOS native buttons with
  `NSButton.BezelStyle.glass` raw value `16`. They are still incomplete because
  `makepad-click` logs alone do not prove native delivery,
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
  delivery. Step 145 fixes the retained CGEvent probe coordinate conversion,
  but post-fix Studio builds `[173]` and `[174]` still did not produce native
  click delivery. Step 151 reruns the global CGEvent probe with Step 150
  `NSWindow.sendEvent:` diagnostics and still does not produce
  `window-send-event`, proving that retained probe does not close the
  physical/system click gate. A physical user/system click is still unproven.
  UIKit controls now have a compiled UIKit installer skeleton from Step 148,
  but no iOS 26 runtime control validation exists yet.
  Step 146 mirrors macOS native button descriptor labels into AppKit
  accessibility labels, and Studio release build `[2]` proves those labels are
  applied at runtime for the aichat probe controls. Focus traversal, VoiceOver
  behavior, duplicate native/Makepad labels, and UIKit accessibility remain
  unresolved. Non-macOS backends remain no-op.
- full native interior Liquid Glass is not implemented; Step 96 proves lower
  scene pass routing, and Step 98 records that the current native interleave
  visual result has no transparency/refraction/liquid distortion.
- Phase H is not implemented.
- Step 134 closes the v4.1 fullscreen fallback/restore probe. Full native glass
  inside fullscreen is still not implemented.
- Step 129 confirms animated spacing updates keep the macOS native batch
  installed, but it does not prove final morphing visual quality.
- Step 135 adds an app-level inactive event source, but inactive transition
  behavior is still unproven until focus can be moved away from the
  Studio-launched child app.
- Step 133 closes the self-driven same-screen geometry snapshot gap, but real
  Stage Manager, split-view, and multi-display geometry behavior remains
  unproven.
- Phase I popup/modal policy is defined in
  `APPLE-NATIVE-POPUP-MODAL-GLASS-POLICY.md`, and Step 136 proves the first
  macOS transient-window native substrate install plus a popup-local draw pass.
  Step 136 also proves synthetic platform-to-app `PopupDismissed` delivery.
  Step 137 proves popup widget-tree descriptor export for the macOS probe path.
  Step 138 proves stable iOS unsupported logging for transient popup probes,
  and Step 142 proves the iOS test target for that gate now compiles with
  `--no-run`.
  Step 143 clarifies the iOS transient unsupported reason as
  `transient-platform-window-missing`, because iOS popups currently render as
  main-window overlay passes rather than separate UIKit platform windows.
  Step 139 implements the macOS Escape route, but physical/system Escape
  validation did not produce logs in the current automation environment.
  Step 140 proves Studio/Makepad outside-click dismissal. Physical AppKit
  outside-click evidence, trusted Escape evidence, UIKit transient
  implementation and runtime validation, and separate-platform-window modal
  native glass remain incomplete. Step 141 covers only in-window Modal
  unsupported observability.
- Runtime switching remains a known limitation.
- Full native glass inside fullscreen remains a known limitation.
- Stage Manager and multi-display behavior remain known limitations.
- Popup/modal native glass remains a separate implementation phase.

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
