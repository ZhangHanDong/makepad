# Step 184 - Guarded Interleave Production Visual Pass

Date: 2026-05-13

## Context

Step 182 made `AppleNativeInterleave` directly runnable behind an explicit
production-preview guard. Step 183 fixed the app-facing success log. The
remaining macOS gate was a system-composited visual pass of the guarded
production preview, not a Studio framebuffer screenshot.

## Manual Evidence

User-provided system screenshot:

```text
/Users/zhangalex/Desktop/截屏2026-05-13 18.07.33.png
```

Expected runtime target:

```text
makepad-example-aichat-apple-native-interleave-production-preview
```

Previously verified build for this target:

```text
build_id=[37]
```

Relevant build `[37]` logs:

```text
[liquid-glass] AppleNativeInterleave guarded production preview enabled; manual visual validation still required
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

Observed screenshot result:

- no diagnostic grid or stripe field is visible
- the native glass treatment covers the broad interior, not only rounded edges
- the lower production scene produces visible color/detail for the native glass
  to sample
- foreground text, controls, and input affordances remain readable above the
  glass stack
- the result looks like a system-composited glass surface rather than a plain
  opaque gray Makepad overlay

## Verdict

The guarded macOS `AppleNativeInterleave` production preview passes its first
system-composited visual gate.

This does not complete the full Apple native Liquid Glass objective. It only
unblocks the next macOS route decision: the guarded preview can now be promoted
to a macOS experimental backend, while iOS 26 runtime validation and advanced
behavior gates remain incomplete.
