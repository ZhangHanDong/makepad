# Step 193 - Interleave Regular Diagnostic Style

Date: 2026-05-15

## Context

Manual feedback on the `apple-native-interleave` diagnostic overlay reported
that the window could still look like a flat opaque grid rather than a Liquid
Glass material. The diagnostic overlay was using the `clear` native style, so
the failure could still be either:

- `clear` is too visually subtle for the current interleave proof
- the native AppKit glass view is not producing visible material/refraction in
  the current Metal sibling hierarchy

Step 193 adds a style-only A/B diagnostic. The lower scene, transparent overlay,
and interleave hierarchy remain the same; only the native panel style changes
from `clear` to `regular`.

## Implementation

New environment override:

```text
AICHAT_NATIVE_INTERLEAVE_STYLE=regular|clear|0|1
```

The default remains `clear`, preserving the existing
`makepad-example-aichat-apple-native-interleave-experimental` behavior.

New Studio runnable:

```text
makepad-example-aichat-apple-native-interleave-regular-diagnostic-overlay
```

The runnable sets:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=diagnostic
AICHAT_NATIVE_INTERLEAVE_STYLE=regular
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

## Verification

Unit test:

```text
cargo test -p makepad-example-aichat --release aichat_native_interleave_style_override_accepts_regular_and_clear
```

Result:

```text
1 passed; 0 failed
```

Release check:

```text
cargo check -p makepad-example-aichat --release
```

Result:

```text
Finished `release` profile [optimized]
```

Studio release build:

```text
build_id=[62]
```

Key logs:

```text
[liquid-glass] native-lower-scene-pass=draw profile=diagnostic proof=Refraction grid_strength=0.180 detail_strength=0.260
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=regular style_raw=0
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

## Manual Decision

This runnable is still a diagnostic view, not a product style. It should show
the diagnostic grid/stripe lower scene.

Interpretation:

- If `regular` shows a noticeably stronger native material while `clear` looks
  flat, production can keep the interleave model and tune/select style.
- If `regular` also looks like a flat grid, the remaining blocker is the
  AppKit/Metal composition model rather than production tint or opacity.

