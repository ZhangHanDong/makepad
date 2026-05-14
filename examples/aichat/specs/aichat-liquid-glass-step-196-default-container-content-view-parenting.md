# Step 196 - Default Container ContentView Parenting

Date: 2026-05-15

## Context

Step 194 introduced `NSGlassEffectContainerView.contentView` parenting as a
guarded diagnostic. Apple documentation makes this the correct public-API
model, not just an experiment:

- `NSGlassEffectContainerView.contentView` is "the view that contains
  descendant views to merge together when in proximity to each other."
- The container elevates descendants of `contentView`, merges similar views
  within `spacing`, and batches similar glass effect views for performance.
- `NSGlassEffectView` embeds its `contentView` in a dynamic glass effect.

References:

- https://developer.apple.com/documentation/appkit/nsglasseffectcontainerview/contentview
- https://developer.apple.com/documentation/appkit/nsglasseffectcontainerview/spacing
- https://developer.apple.com/documentation/appkit/nsglasseffectview

Therefore, the v4.1 macOS native batch installer should parent panel views
under the container content view by default. Direct container parenting remains
available only as a diagnostic fallback.

## Implementation

`MAKEPAD_NATIVE_GLASS_USE_CONTAINER_CONTENT_VIEW` now defaults to enabled.

Truthy/default values use contentView parenting:

```text
unset
""
1
true
on
content-view
```

Explicit fallback values use the old direct container parenting path:

```text
0
false
off
direct
```

When enabled, the installer creates an `NSView`, sets it as the
`NSGlassEffectContainerView.contentView` when the default content view is nil,
and then adds native `NSGlassEffectView` panels under that content view.

## Verification

Release checks:

```text
cargo check -p makepad-platform --release
cargo test -p makepad-platform --release native_glass_container_content_view_env_defaults_to_content_view_parenting
cargo test -p makepad-platform --release native_glass
```

Results:

```text
native_glass_container_content_view_env_defaults_to_content_view_parenting ... ok
native_glass: 44 passed; 0 failed
```

Studio release build:

```text
build_id=[69]
target=makepad-example-aichat-apple-native-interleave-experimental
```

Key logs:

```text
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] native-container-panel-parent container=0000000000000010 requested=contentView role=container-created-contentView class=NSView
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

## Remaining Gate

This fixes the public API parenting model for macOS native containers. It does
not by itself prove that the current interleave composition produces sufficient
visual Liquid Glass material. Manual system-composited visual validation is
still required because Studio screenshots omit the AppKit native layer and
system `screencapture` remains black on this machine.

