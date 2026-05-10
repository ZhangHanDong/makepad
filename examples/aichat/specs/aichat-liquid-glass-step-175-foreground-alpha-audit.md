# Step 175 - Interleave Foreground Alpha Audit

Date: 2026-05-10

## Context

After Step 174 fixed Studio screenshot routing, the
`interleave-transparent-overlay` screenshot showed the aichat foreground UI over
an apparently black background. Since the aichat window pass is configured as
transparent, this needed an alpha audit before treating the black pixels as an
opaque primary-surface blocker.

## Evidence

Input screenshot:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-28-kind-0-req-2753-1778411287580.png
```

Alpha histogram check:

```text
size (900, 700)
zero_alpha 589869
nonzero_alpha 40131
opaque_alpha 1706
min_alpha 0
max_alpha 255
```

The screenshot has 630,000 total pixels. 589,869 pixels are fully transparent.

## Verdict

The black-looking Studio screenshot background is mostly transparent primary
surface, not a full opaque black pass clear.

This proves the current
`makepad-example-aichat-macos-native-clear-interleave-transparent-overlay` probe
is close to a foreground-only upper Makepad surface: the primary framebuffer
mostly contains transparent pixels plus text and controls.

This still does not prove production `AppleNativeInterleave`. Native AppKit
material visibility must be judged through manual/system composition because
Studio framebuffer screenshots do not include the AppKit glass layer itself.
