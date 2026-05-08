# aichat Liquid Glass v4.2 Native Interleave Visual Verdict

This checklist is not completed yet. It defines the human visual gate for the
Step 96 `lower-scene-pass` prototype.

## Run Targets

Use Makepad Studio remote release runs only.

- [ ] Baseline: `makepad-example-aichat-macos-native-clear`
- [ ] Candidate:
  `makepad-example-aichat-macos-native-clear-interleave-lower-scene-pass`

## Required Logs

Before judging visuals, confirm the candidate run logs all of:

- [ ] `LowerScenePass`
- [ ] `native-lower-scene-pass=draw`
- [ ] `native-interleave-layer-probe lower-scene-role=draw`
- [ ] `state=4 substrate=macos-native style=clear style_raw=1`
- [ ] `panels_installed=4 panels_failed=0`

Step 96 already produced these logs in a Studio release run. Reconfirm them
after any renderer or platform change.

## What Must Be Visible

The candidate should show the aichat lower scene across the window. That alone
is not enough for a pass verdict.

Pass verdict:

- [ ] Glass panel interiors visibly sample the lower scene.
- [ ] The sampled lower scene shows recognizable native blur, refraction, or
  liquid distortion inside panel interiors, not only on panel edges.
- [ ] Resizing the window keeps the lower scene and glass treatment aligned.
- [ ] Text and controls remain above the material and readable.
- [ ] Pointer, text input, and scrolling remain routed through Makepad.

Fail verdict:

- [ ] The candidate is only a flat opaque background behind the UI.
- [ ] Glass treatment is visible only at edges or rounded corners.
- [ ] There is no recognizable blur/refraction/liquid distortion over the
  lower scene.
- [ ] Text/control input is blocked or AppKit consumes pointer events.

If the fail verdict matches what is visible, do not claim full native Liquid
Glass. Keep `AppleNativeInterleave` as a prototype and continue full interior
visual work on the ShaderBackdrop or hybrid route.

## Comparison Notes

- [ ] Compare against `makepad-example-aichat-macos-native-clear`; the candidate
  should not merely look like the same underlay with an opaque scene inserted.
- [ ] Compare panel interiors, not only the outer shell edge.
- [ ] Check both bright and dark desktop wallpaper if the window remains
  transparent enough for wallpaper contribution.
- [ ] Capture screenshots for the baseline and candidate after the same initial
  draw settles.

## Current Evidence

The latest Step 96 Studio screenshot for `lower-scene-pass` shows the lower
scene gradient/grid framebuffer. This proves lower scene pass rendering and
routing. It does not prove final user-visible native composition or native
Liquid Glass sampling quality, because Studio screenshots do not capture the
full native overlay stack.
