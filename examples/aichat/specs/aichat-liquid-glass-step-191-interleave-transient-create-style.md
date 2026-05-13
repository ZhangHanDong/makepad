# Step 191 - Interleave Transient Create Style

Date: 2026-05-13

## Context

Step 189 proved that the experimental macOS `AppleNativeInterleave` backend can
install native glass for transient popup widget descriptors and can exercise
the dismiss path. However, the popup platform-window create path still emitted
an early misleading line:

```text
[liquid-glass] transient-window=popup state=Rejected reason=backend-not-native
```

The later popup-local batch still installed, but the early create-path log made
the interleave transient state look partially rejected.

## Root Cause

`requested_native_glass_style_from_env()` intentionally maps only direct
single-window native substrate backends:

```text
macos-native
macos-native-clear
apple-native-underlay
apple-native-underlay-clear
auto
```

It intentionally does not map:

```text
apple-native-interleave
```

That is correct for main-window creation because interleave is installed later
through native panel batches, not through the legacy single substrate create
path. The transient popup create probe reused the main-window resolver, so it
mistook the interleave backend for a non-native backend.

## Implementation

The macOS style resolver is now split:

- `requested_native_glass_style_from_value` keeps the main-window create-path
  mapping unchanged.
- `requested_transient_native_glass_style_from_value` preserves all existing
  direct native mappings and additionally maps `apple-native-interleave` to
  `MacosNativeGlassStyle::Clear`.

The popup create probe now uses the transient resolver.

## Verification

Commands:

```text
cargo test -p makepad-platform --release native_glass
cargo check -p makepad-platform --release
```

Result:

```text
43 passed; 0 failed
cargo check finished release profile
```

Studio release build:

```text
build_id=[59]
runnable=makepad-example-aichat-apple-native-interleave-transient-probe
```

Key logs:

```text
[liquid-glass] AppleNativeInterleave macOS experimental backend enabled; iOS and advanced behavior still require validation
[liquid-glass] native-lower-scene-pass=draw profile=production proof=InteriorNoChroma grid_strength=0.000 detail_strength=0.220
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] transient-window-probe=request-open parent=WindowId(0, 0) popup=WindowId(1, 0)
[liquid-glass] transient-window=popup state=Installed substrate=macos-native style=clear reason=installed-on-proofed-hierarchy
[liquid-glass] transient-window-probe=widget-tree popup_size=(240.0,160.0) panels=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=1 panels_failed=0
```

The earlier `transient-window=popup state=Rejected reason=backend-not-native`
line is gone for the interleave transient probe.

## Verdict

The macOS interleave transient create path now reports the same native style as
the active interleave backend instead of a misleading backend rejection. This
does not complete popup/modal native glass as a whole; UIKit transient runtime
validation and modal platform-window policy remain separate Phase I gates.
