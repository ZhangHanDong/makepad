# Step 174 - Native Interleave Screenshot Routing

Date: 2026-05-10

## Context

Step 173 added
`makepad-example-aichat-macos-native-clear-interleave-transparent-overlay`.
The first Studio screenshot taken from build `[27]` showed only the lower-scene
grid and no foreground UI, even though `WidgetTreeDump` still reported the full
aichat widget tree.

That proved a Studio gate problem: lower-scene `Drawable` passes were consuming
the pending Studio screenshot request before the primary window pass could use
it.

## Implementation

`platform/src/os/apple/macos/macos.rs` now draws non-primary lower-scene
diagnostic window passes without consuming `screenshot_requests`. The request is
temporarily removed before the lower-scene draw and restored immediately after
that draw returns, so the primary window pass remains the screenshot source.

This applies to the current lower-scene role and lower-scene mirror probe paths.

## Verification

```text
cargo check -p makepad-platform --release
```

Result:

```text
Finished `release` profile [optimized] target(s) in 2.66s
```

Studio remote release rerun:

```text
RunItem makepad-example-aichat-macos-native-clear-interleave-transparent-overlay
build_id=[28]
```

The run still reached the native interleave gates:

```text
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] compositing-proof=transparent-overlay
[liquid-glass] native-lower-scene-pass=draw
[liquid-glass] native-interleave-layer-probe lower-scene-role=draw
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer frame=(3.0,3.0,894.0,694.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
```

Studio screenshot after the fix:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-28-kind-0-req-2753-1778411287580.png
```

The screenshot shows the primary aichat foreground UI instead of the lower-scene
grid-only image.

## Verdict

The Studio screenshot gate for the current macOS interleave probes is fixed:
lower-scene diagnostic passes no longer steal the default screenshot request
from the primary foreground pass.

This does not prove production `AppleNativeInterleave`, because Studio
framebuffer screenshots still cannot show native AppKit glass composition. It
does restore reliable Studio evidence for the Makepad foreground surface while
manual/system screenshots remain required for native material visibility.
