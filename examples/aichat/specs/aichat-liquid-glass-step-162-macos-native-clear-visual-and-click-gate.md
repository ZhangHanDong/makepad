# Step 162 - macOS Native Clear Visual and Click Gate

Date: 2026-05-10

## Context

This step records the latest Studio-run `macos-native-clear` control probe after
the user supplied a desktop screenshot for visual inspection.

The screenshot under review was:

```text
/Users/zhangalex/Desktop/截屏2026-05-09 18.15.46.png
```

## Runtime Run

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-control-probe
build_id=[4]
```

The build was launched through the Studio remote bridge after clearing the
previous control probe build. The Studio framebuffer screenshot for build `[4]`
was captured at:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-4-kind-0-req-401-1778354811559.png
```

That framebuffer capture shows the app render target only; it does not include
desktop compositor transparency, so it is not useful as visual proof of the
native glass effect. The user desktop screenshot is the relevant visual
evidence for compositor glass.

## Install Evidence

Build `[4]` reached the native clear substrate and native-control install gates:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-underlay state=Installed reason=installed-on-proofed-hierarchy
[liquid-glass] backend=apple-native-controls state=Installed reason=installed-appkit-buttons controls_total=2 controls_visible=2
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] container=0000000000000010 state=Installed reason=installed panels_installed=4 panels_failed=0
```

The two native control mirrors were visible AppKit siblings:

```text
[liquid-glass] native-control-frame control=0000000000000043 kind=Button { role: Default } label="Clear" makepad=(695.5,190.0,78.0,36.0) appkit=(695.5,474.0,78.0,36.0) z_order=0 visible=true
[liquid-glass] backend=apple-native-controls button-style control=0000000000000043 label="Clear" bezel=glass raw=16
[liquid-glass] backend=apple-native-controls accessibility-label control=0000000000000043 label="Clear"
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000043 label="Clear" point=(734.5,492.0) result_class=NativeGlassButton matches_control=true
[liquid-glass] native-control-frame control=0000000000000044 kind=Button { role: Default } label="↑" makepad=(781.5,190.0,44.0,44.0) appkit=(781.5,466.0,44.0,44.0) z_order=0 visible=true
[liquid-glass] backend=apple-native-controls button-style control=0000000000000044 label="↑" bezel=glass raw=16
[liquid-glass] backend=apple-native-controls accessibility-label control=0000000000000044 label="↑"
[liquid-glass] backend=apple-native-controls event=appkit-hit-test-probe control=0000000000000044 label="↑" point=(803.5,488.0) result_class=NativeGlassButton matches_control=true
```

The hierarchy also showed the native controls above the Metal view:

```text
[liquid-glass] backend=apple-native-controls hierarchy index=1 role=metal-view class=RenderViewClass frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] backend=apple-native-controls hierarchy index=2 role=native-control class=NativeGlassButton frame=(695.5,474.0,78.0,36.0) hidden=false
[liquid-glass] backend=apple-native-controls hierarchy index=3 role=native-control class=NativeGlassButton frame=(781.5,466.0,44.0,44.0) hidden=false
```

## Visual Verdict

The user desktop screenshot is consistent with the current expected
`macos-native-clear` result:

- window-wide compositor transparency, blur, and tint are visible;
- rounded-window edges show stronger liquid refraction/highlight behavior;
- interior regions mostly show native blur/tint, not the full-area shader-like
  liquid warp used by the debug backdrop probes.

This is treated as a valid visual pass for the current macOS native underlay
path, not as proof of full native interactive controls or full interior liquid
distortion.

## Click Gate

After install log index `118`, the live log query for:

```text
window-send-event|button-mouse-down|target-action|button-action|native-control-probe=makepad-click|metal-view-mouse-down|metal-view-hit-test
```

returned no entries. This means no physical/trusted click had reached the
diagnostic chain at the time of this note.

The remaining macOS native-control gate is still a physical/trusted click on
the visible `Clear` button that logs:

```text
window-send-event -> button-mouse-down -> target-action -> button-action -> native-control-probe=makepad-click
```

## Verification Commands

```text
git diff --check
```
