# aichat Liquid Glass v4.1 Manual Visual Checklist

This checklist is intentionally unchecked. It is the remaining human validation
work for macOS 26 native Liquid Glass visibility and readability.

## Setup

- [ ] Start Makepad Studio desktop with remote control enabled.
- [ ] Use a release run through Makepad Studio, not raw `cargo run`.
- [ ] Before rerunning the same target, clear the previous Studio build tab.
- [ ] Capture at least one screenshot for every failed visual item.

Required Studio run targets:

- [ ] `makepad-example-aichat-macos-native`
- [ ] `makepad-example-aichat-macos-native-clear`

Optional comparison targets:

- [ ] shader/default aichat target
- [ ] unsupported-runtime or non-macOS fallback, if available

Current AppleNativeInterleave targets:

- [ ] `makepad-example-aichat-apple-native-interleave-experimental`
- [ ] `makepad-example-aichat-apple-native-interleave-diagnostic-overlay`

## Startup Logs

- [ ] `macos-native` reaches a State 4 compatibility log:

```text
[liquid-glass] state=4 substrate=macos-native style=regular style_raw=0
```

- [ ] `macos-native-clear` reaches a State 4 compatibility log:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
```

- [ ] v4.1 native container log reports one container:

```text
[liquid-glass] native-container state=installed platform=macos containers=1 panels_installed=M panels_failed=0
```

- [ ] v4.1 native-panel logs exist for shell, sidebar, main area, and composer.
- [ ] `panels_failed` stays `0` on a supported macOS 26 runtime.

## Native Visibility

- [ ] Shell background shows native glass across the interior, not only at the
  outer edge.
- [ ] Sidebar native glass is visible but does not overpower file/chat labels.
- [ ] Main area native glass is visible behind Markdown and generated content.
- [ ] Composer native glass is visible behind input controls.
- [ ] `macos-native-clear` is visually distinguishable from `macos-native`.
- [ ] Shader/default mode still looks like the existing Makepad fallback.

## Readability

- [ ] Bright wallpaper: sidebar text remains readable.
- [ ] Bright wallpaper: main conversation text remains readable.
- [ ] Bright wallpaper: composer placeholder/input text remains readable.
- [ ] Dark wallpaper: sidebar text remains readable.
- [ ] Dark wallpaper: main conversation text remains readable.
- [ ] Dark wallpaper: composer placeholder/input text remains readable.
- [ ] High-contrast areas do not produce excessive double highlights.
- [ ] Halo/noise does not distract from text.

## Geometry

- [ ] The red-box rounded-corner regression area stays rounded, with no stray
  right-angle patch.
- [ ] Window resize keeps shell/sidebar/main/composer native panels aligned with
  Makepad-rendered content.
- [ ] Panel edges do not drift after repeated resize.
- [ ] The resize grip remains usable and visible.
- [ ] Rounded corners look acceptable on both regular and clear native styles.

## Input

- [ ] Sidebar buttons remain clickable.
- [ ] Text input accepts typing.
- [ ] Return submits a chat message.
- [ ] Scroll works in the conversation view.
- [ ] Native glass panels do not intercept input.
- [ ] Window drag behavior still works in expected drag regions.

## Splash And AI-Generated UI

- [ ] A generated `runsplash` block with a full-size opaque root does not cover
  the native glass completely.
- [ ] Splash opaque-root guard logs when it clamps an opaque root.
- [ ] Non-Splash fenced code blocks are not modified by the guard.
- [ ] Generated buttons remain clickable after native mode is active.

## Fallback And Limitations

- [ ] Shader/default mode does not emit native container install success logs.
- [ ] Unsupported runtime fallback does not crash, if testable.
- [ ] Runtime switching is not tested as supported behavior.
- [ ] Fullscreen, Stage Manager, multiple displays, and popup/modal glass are
  treated as known limitations for this pass.

## Result Summary

- [ ] Attach or link screenshots for any failed native visibility item.
- [ ] Record whether regular style is acceptable.
- [ ] Record whether clear style is acceptable.
- [ ] Record whether any visual tuning changes are required before Phase F.

## AppleNativeInterleave Diagnostic Decision

Use the diagnostic overlay before changing production tuning:

- [ ] Run `makepad-example-aichat-apple-native-interleave-diagnostic-overlay`.
- [ ] Confirm startup logs include:

```text
[liquid-glass] native-lower-scene-pass=draw profile=diagnostic proof=Refraction
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed
```

- [ ] If diagnostic overlay shows visible grid/refraction/liquid distortion but
  `makepad-example-aichat-apple-native-interleave-experimental` looks flat,
  treat the next task as production visual tuning.
- [ ] If diagnostic overlay also looks black, opaque, or edge-only, treat the
  next task as a native/macOS composition visibility blocker.
- [ ] Do not use Studio framebuffer screenshots alone for this decision; they
  omit the AppKit native glass layer and normally show only the transparent
  upper Makepad surface.
- [ ] Capture a system-composited screenshot or record a manual verdict for the
  diagnostic run before changing production parameters.

## Validation Update 2026-05-02

Studio release validation reached the native backend on macOS 26 for both
required targets:

- `makepad-example-aichat-macos-native`
  - compatibility log reached `state=4 substrate=macos-native style=regular style_raw=0`
  - v4.1 batch log reported `containers=1 panels_installed=4 panels_failed=0`
- `makepad-example-aichat-macos-native-clear`
  - compatibility log reached `state=4 substrate=macos-native style=clear style_raw=1`
  - v4.1 batch log reported `containers=1 panels_installed=4 panels_failed=0`

The user reported the manual visual pass as acceptable after seeing the Studio
run. Follow-up tuning is still required to make the native material more visible
across wallpapers by reducing Makepad overlay tint, highlight, noise, and halo.
After Step 11 tuning, both Studio targets were rerun through fresh release
RunItems and still reached State 4 with four installed panels. The repeated
native batch install log flood was no longer observed during the validation
wait after startup.

## Compositing Proof Update 2026-05-02

`makepad-example-aichat-macos-native-clear-proof-stripes` reached State 4 with
the striped proof substrate and transparent Makepad overlay enabled. The user
confirmed that colored stripes were visible and that the stripe coverage followed
window resizing after the proof substrate was widened.

The user could not visually distinguish the interior as glass material; the
result appeared as flat colored stripes. This means v4.1 proves native underlay
visibility, but it does not prove complete Liquid Glass inside the panels. The
current under-Metal model remains insufficient for full Liquid Glass and should
feed into the v4.2 compositing design.

## AppleNativeInterleave Update 2026-05-13

`makepad-example-aichat-apple-native-interleave-experimental` is now the macOS
experimental backend for the full native interleave route. It uses a production
lower scene and keeps the upper Makepad surface mostly transparent.

`makepad-example-aichat-apple-native-interleave-diagnostic-overlay` is the
high-contrast manual visual gate. It runs the same `apple-native-interleave`
backend but switches the lower scene to the diagnostic `Refraction` profile.
Use it to decide whether a failed visual report is caused by subtle production
tuning or by native composition visibility.
