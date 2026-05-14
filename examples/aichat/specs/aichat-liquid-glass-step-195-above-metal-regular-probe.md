# Step 195 - Above-Metal Regular Probe

Date: 2026-05-15

## Context

Steps 193 and 194 added interleave diagnostics for `clear`, `regular`, and
container `contentView` parenting. If those still look like flat lower-scene
content, the next isolation point is whether `NSGlassEffectView` itself becomes
visibly glassy when it is placed above the Makepad Metal view.

The above-metal probe is not a production architecture because it places native
AppKit glass above Makepad-rendered content. It is a diagnostic to separate:

- native glass material/API visibility
- the v4.1 interleave compositing model where Makepad content remains above
  native panels

## Implementation

Existing diagnostic:

```text
makepad-example-aichat-macos-native-above-metal-probe
```

uses:

```text
AICHAT_NATIVE_ABOVE_METAL_PROBE=clear
AICHAT_METAL_PROBE_PATTERN=moving-pattern
```

Step 195 adds a regular-style comparison target:

```text
makepad-example-aichat-macos-native-above-metal-regular-probe
```

with:

```text
AICHAT_NATIVE_ABOVE_METAL_PROBE=regular
AICHAT_METAL_PROBE_PATTERN=moving-pattern
```

## Verification

Studio release build:

```text
build_id=[68]
```

Key log:

```text
[liquid-glass] above-metal-probe state=installed style=regular style_raw=0 input=diagnostic-overlay
```

## Manual Decision

- If the above-metal regular probe is visibly glassy while the interleave
  diagnostics are flat, the remaining blocker is the production compositing
  model: Makepad content needs to stay visually above native glass without
  preventing native glass from sampling useful content.
- If the above-metal regular probe is also flat, the blocker is likely native
  API/material configuration or runtime behavior rather than the specific
  interleave hierarchy.

