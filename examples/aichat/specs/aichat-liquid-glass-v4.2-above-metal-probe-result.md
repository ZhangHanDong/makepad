# aichat Liquid Glass v4.2 Above-Metal Probe Result

## Summary

The above-Metal probe is implemented as a dedicated Studio runnable:

```text
makepad-example-aichat-macos-native-above-metal-probe
```

It launches a transparent aichat window, draws a high-contrast Makepad Metal
pattern, and installs one diagnostic `NSGlassEffectView` above the Makepad
Metal view with `AICHAT_NATIVE_ABOVE_METAL_PROBE=clear`.

## Runtime Evidence

Studio release run:

```text
BuildStarted build_id=[13] package=makepad-example-aichat-macos-native-above-metal-probe
Finished `release` profile [optimized]
[liquid-glass] above-metal-probe state=installed style=clear style_raw=1 input=diagnostic-overlay
```

The dedicated target uses:

```text
AICHAT_NATIVE_ABOVE_METAL_PROBE=clear
AICHAT_METAL_PROBE_PATTERN=moving-pattern
```

## Visual Evidence

Studio framebuffer screenshot:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-13-kind-0-req-193-1777955818164.png
```

This screenshot showed only the Makepad Metal pattern. That is expected: Studio
captures the app framebuffer and may not include native AppKit overlays placed
above the Metal view.

System screenshot after bringing the `makepad-example-aichat` process front:

```text
/tmp/aichat-above-metal-process-front.png
```

This screenshot showed the Makepad Metal pattern plus a large rounded native
glass rectangle above it. The native rectangle was not present in the Studio
framebuffer capture, which confirms a Studio visibility limitation for
above-Metal AppKit overlays.

Studio widget dump for the same build remained Makepad-scoped:

```text
W3 4
0 -1 main_window Window 0 0 900 700
1 0 body KeyboardView 0 0 900 700
2 1 metal_probe_pattern View 3 3 894 694
3 1 resize_grip Vector 3 3 34 34
```

The native AppKit glass view is intentionally not a Makepad widget.

## Result

The probe distinguishes the two relevant cases:

- AppKit native glass above the Makepad Metal view can be visible in real
  macOS window composition.
- Studio framebuffer screenshots alone cannot prove or disprove that native
  overlay.

This is not a production aichat hierarchy. The native glass view sits above the
Makepad Metal view and can cover Makepad-rendered text and controls. Full
native Liquid Glass would require a real interleave architecture with separate
Makepad-rendered lower and upper surfaces.
