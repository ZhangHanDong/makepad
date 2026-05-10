# Step 172 - Native Interleave Hierarchy Probe

Date: 2026-05-10

## Context

Step 171 records why `AppleNativeUnderlay` only reveals native material at
edges. The next native-interleave question is whether the AppKit view/layer
hierarchy is still wrong, or whether the remaining blocker is Makepad render
pass splitting.

## Implementation

`platform/src/os/apple/macos/macos_window.rs` now logs the AppKit subview and
layer order when `AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE` is set and native glass
installs.

The log is probe-gated and does not affect normal native glass runs.

## Runtime Evidence

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-interleave-lower-scene-pass
build_id=[24]
```

The run reached the existing lower-scene and native-glass gates:

```text
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] native-lower-scene-pass=draw
[liquid-glass] native-interleave-layer-probe drawable=available
[liquid-glass] native-interleave-layer-probe lower-scene-role=draw
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The new hierarchy evidence:

```text
[liquid-glass] native-interleave-hierarchy reason=native-container-installed subviews=3
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer frame=(3.0,3.0,894.0,694.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
```

## Verification

```text
cargo check -p makepad-platform --release
```

Result:

```text
Finished `release` profile [optimized] target(s) in 3.13s
```

## Verdict

The macOS probe hierarchy already has the required native order for a first
interleave experiment:

```text
lower Makepad Metal sibling
native NSGlassEffectContainerView
primary Makepad Metal view
```

The current blocker is therefore not simply AppKit sibling ordering. The
remaining production blocker is render-pass ownership: the primary Makepad
surface still paints the full aichat UI and readability/interior layers above
native glass. A production `AppleNativeInterleave` needs a real foreground split
so the upper surface contains only transparent foreground text/controls and the
background/interior content lives on the lower surface.
