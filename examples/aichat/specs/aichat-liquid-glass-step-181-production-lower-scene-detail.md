# Step 181 - Production Lower Scene Detail

Date: 2026-05-12

## Context

Step 179 introduced the production interleave preview by removing the diagnostic
grid from the lower scene. That made the preview less artificial, but also
reduced the amount of visible detail available for native glass to sample and
refract.

This step keeps the production preview grid-free while adding a separate
non-grid lower-scene detail control:

```text
scene_detail_strength
```

## Implementation

`DrawAichatBackdropScene` now renders a procedural ripple/grain detail term
separately from the diagnostic grid. The production native interleave profile
uses:

```text
grid_strength=0.000
detail_strength=0.220
```

The diagnostic profile keeps stronger detail:

```text
detail_strength=0.260
```

This gives native glass more real background detail to sample without
reintroducing the visible grid/stripe diagnostic artifact.

## Verification

Commands:

```text
cargo test -p makepad-example-aichat --release aichat_native_interleave
cargo check -p makepad-example-aichat --release
```

Result:

```text
2 passed; 0 failed
cargo check finished release profile
```

Studio release build:

```text
build_id=[33]
```

Key logs:

```text
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

Studio screenshot path:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-33-kind-0-req-3025-1778597606830.png
```

Foreground alpha histogram remains unchanged from the prior production preview:

```text
size=(900, 700)
zero_alpha=589869
nonzero_alpha=40131
opaque_alpha=1706
partial_alpha=38425
```

## Verdict

The production preview now has grid-free lower-scene detail for native glass to
sample. This improves the next manual visual gate without promoting
`AppleNativeInterleave` to a default backend.

The remaining requirement is still a system-composited manual verdict: the user
must confirm whether build `[33]` shows recognizable native glass material and
liquid/refraction behavior without diagnostic grid artifacts.
