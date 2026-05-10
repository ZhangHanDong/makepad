# Step 177 - Native Interleave Diagnostic Overlay

Date: 2026-05-10

## Context

Step 176 left the native AppKit material visual gate blocked on manual/system
composition evidence. The previous transparent-overlay runnable used the subtle
`Interior` lower-scene profile, which may be too weak for a manual sampling
verdict through native glass.

## Implementation

`examples/aichat/src/main.rs` now supports:

```text
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=diagnostic
```

Supported values:

- default / unknown -> `ShaderBackdropProof::Interior`
- `diagnostic`, `refraction`, `strong` -> `ShaderBackdropProof::Refraction`
- `interior-no-chroma`, `no-chroma` -> `ShaderBackdropProof::InteriorNoChroma`

`makepad.splash` adds:

```text
makepad-example-aichat-macos-native-clear-interleave-diagnostic-overlay
```

The target combines:

- native clear glass
- lower-scene pass
- transparent `GlassPanel` overlays
- stronger `Refraction` lower-scene profile

## Verification

Focused test:

```text
cargo test -p makepad-example-aichat --release aichat_native_interleave_scene_profile_accepts_diagnostic_values
```

Result:

```text
test tests::aichat_native_interleave_scene_profile_accepts_diagnostic_values ... ok
```

Release check:

```text
cargo check -p makepad-example-aichat --release
```

Result:

```text
Finished `release` profile [optimized] target(s) in 27.44s
```

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-interleave-diagnostic-overlay
build_id=[30]
```

Key logs:

```text
[liquid-glass] native-interleave-layer-probe state=installed hidden=0 mode=LowerScenePass placement=sibling-below-primary
[liquid-glass] compositing-proof=transparent-overlay
[liquid-glass] native-lower-scene-pass=draw profile=Refraction
[liquid-glass] native-interleave-layer-probe lower-scene-role=draw
[liquid-glass] native-interleave-hierarchy index=0 role=metal-sibling view_class=NSView layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=1 role=native-glass-container view_class=NSGlassEffectContainerView layer_class=NSViewBackingLayer frame=(3.0,3.0,894.0,694.0) hidden=false
[liquid-glass] native-interleave-hierarchy index=2 role=primary-metal-view view_class=RenderViewClass layer_class=CAMetalLayer frame=(0.0,0.0,900.0,700.0) hidden=false
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

Studio foreground screenshot:

```text
/var/folders/rj/fpdb5j3d71v4h0464cs2xn500000gn/T/makepad_studio_hub/build-30-kind-0-req-2833-1778411977919.png
```

Alpha histogram:

```text
size (900, 700)
zero_alpha 589869
nonzero_alpha 40131
opaque_alpha 1706
min_alpha 0
max_alpha 255
```

The diagnostic target therefore preserves the foreground-only property from
Step 175 while switching the lower scene to the stronger `Refraction` profile.

## Verdict

The diagnostic overlay target is now the preferred manual visual gate for the
native interleave prototype because the lower scene is intentionally stronger
and easier to recognize if AppKit glass samples it.

This still does not prove production `AppleNativeInterleave`. It creates a
clearer manual verdict target after system screenshot automation produced an
unusable black frame.
