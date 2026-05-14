# Step 194 - Interleave Container ContentView Probe

Date: 2026-05-15

## Context

Step 193 separated the native style variable by adding a `regular` diagnostic
target. Runtime introspection then showed that `NSGlassEffectContainerView`
responds to `contentView` and `setContentView:`. The current v4.1 installer
adds `NSGlassEffectView` panels directly to the container view. If the AppKit
container expects child glass views to live under `contentView`, direct
subviews can still log as installed while missing part of the native container
semantics.

Step 194 adds a guarded experiment that creates a container content view and
parents native panels there.

## Runtime Introspection

Local SDK headers are not sufficient for macOS 26 API coverage on this machine:

```text
MacOSX15.5.sdk
iPhoneOS18.5.sdk
```

Runtime selector dump on the current OS reported:

```text
NSGlassEffectView:
  cornerRadius
  setCornerRadius:
  setTintColor:
  contentView
  setStyle:
  style
  setContentView:
  tintColor

NSGlassEffectContainerView:
  spacing
  setSpacing:
  contentView
  setContentView:
```

Private selectors such as `_contentLensing` and `_groupIdentifier` were also
observed but remain out of scope for this public-API route.

## Implementation

New guarded platform env:

```text
MAKEPAD_NATIVE_GLASS_USE_CONTAINER_CONTENT_VIEW=content-view
```

When enabled, the macOS native batch installer:

1. asks the `NSGlassEffectContainerView` for `contentView`
2. creates an `NSView` and calls `setContentView:` if the default content view
   is nil
3. adds native panel views to that content view instead of directly to the
   container
4. logs the selected parent role and class

The default path remains unchanged.

New Studio runnable:

```text
makepad-example-aichat-apple-native-interleave-content-view-diagnostic-overlay
```

The runnable sets:

```text
AICHAT_GLASS_BACKEND=apple-native-interleave
AICHAT_NATIVE_INTERLEAVE_LAYER_PROBE=lower-scene-pass
AICHAT_NATIVE_INTERLEAVE_SCENE_PROFILE=diagnostic
AICHAT_NATIVE_INTERLEAVE_STYLE=regular
MAKEPAD_NATIVE_GLASS_USE_CONTAINER_CONTENT_VIEW=content-view
AICHAT_NATIVE_COMPOSITING_PROOF=transparent-overlay
```

## Verification

Release checks:

```text
cargo check -p makepad-platform --release
cargo test -p makepad-platform --release native_glass_container_content_view_env_accepts_truthy_values
cargo test -p makepad-platform --release native_glass
```

Results:

```text
native_glass_container_content_view_env_accepts_truthy_values ... ok
native_glass: 44 passed; 0 failed
```

Studio release build:

```text
build_id=[65]
```

Key logs:

```text
[liquid-glass] native-lower-scene-pass=draw profile=diagnostic proof=Refraction grid_strength=0.180 detail_strength=0.260
[liquid-glass] native-container-panel-parent container=0000000000000010 requested=contentView role=container-created-contentView class=NSView
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
[liquid-glass] state=4 substrate=macos-native style=regular style_raw=0
[liquid-glass] app-substrate=apple-native-interleave state=Installed reason=installed-native-glass-batch
```

## Manual Decision

This target is still diagnostic and intentionally shows the lower-scene grid.

Interpretation:

- If this content-view target shows stronger native material than Step 193,
  the production installer should consider adopting the contentView parenting
  model after input and geometry regression checks.
- If it still looks like flat lower-scene content, the blocker is deeper than
  direct-vs-contentView parenting and should move to AppKit/Metal composition
  model investigation.

