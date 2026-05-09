# Step 145 - macOS Native Control CGEvent Coordinate Fix

Date: 2026-05-10

## Change

The macOS native-control CGEvent probe now converts the AppKit screen point
from bottom-left screen coordinates into the Quartz event coordinate space
before posting `CGEventCreateMouseEvent`.

The probe log now includes both coordinate spaces:

```text
[liquid-glass] backend=apple-native-controls event=cg-event-probe ... screen=(x,y) cg=(x,y)
```

This targets the remaining system-click validation gate. Earlier CGEvent probes
posted the AppKit `convertRectToScreen:` point directly; those builds logged
the probe but did not produce `button-mouse-down`, `target-action`, or
`button-action`.

## Verification

Passed:

```text
cargo test -p makepad-platform native_glass_cg_event_point --release
cargo check -p makepad-platform --release
git diff --check
```

Studio release runs:

```text
RunItem makepad-example-aichat-macos-native-clear-control-cgevent-probe
build_id=[173]

RunItem makepad-example-aichat-macos-native-clear-control-cgevent-pid-probe
build_id=[174]
```

Both runs logged the corrected coordinate conversion:

```text
screen=(2004.5,1024.0) cg=(2004.5,416.0)
```

Both runs still produced no subsequent `button-mouse-down`, `target-action`,
`button-action`, or `native-control-probe=makepad-click` logs.

## Verdict

This fixes the coordinate-space mismatch in the retained CGEvent diagnostic
path. It does not close physical/system click delivery: builds `[173]` and
`[174]` show that the corrected CGEvent points are posted, but macOS still does
not deliver them to the native button in the current automation/security
context.
