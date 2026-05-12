# Step 178 - Diagnostic Interleave Visual Pass

Date: 2026-05-12

## Context

Step 177 added the preferred manual visual gate:

```text
makepad-example-aichat-macos-native-clear-interleave-diagnostic-overlay
```

That target combines:

- lower Makepad Metal sibling
- native `NSGlassEffectContainerView`
- transparent primary Makepad foreground
- stronger `Refraction` lower-scene profile

## Manual Evidence

User-provided system screenshot:

```text
/Users/zhangalex/Desktop/截屏2026-05-12 22.27.13.png
```

Observed result:

- the lower diagnostic grid/color field is visible across the full interior of
  the aichat window
- the effect is not limited to rounded edges or halos
- foreground text and controls remain visible above the glass/interleave stack
- the screenshot shows the desktop around the transparent border, proving this
  is a real system-composited view rather than only a Studio framebuffer image

## Verdict

The diagnostic `AppleNativeInterleave` visual hypothesis passes: with a lower
Makepad Metal scene, native AppKit glass in the middle, and a mostly transparent
primary Makepad foreground, the user can see broad interior glass over the
lower scene.

This does not make production `AppleNativeInterleave` complete yet. The next
implementation slice is to replace the diagnostic lower-scene grid with a real
aichat lower scene / background split while keeping foreground text, controls,
input, widget queries, and Studio screenshot behavior on the upper surface.
