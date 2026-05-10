# Step 170 - Multi-Display Environment Blocker

Date: 2026-05-10

## Context

The Phase H multi-display gate requires moving a native-glass window between
displays and proving:

- descriptor geometry remains in Makepad logical units
- AppKit frame conversion updates correctly
- backing-scale changes are detected when displays differ
- native panels remain installed and aligned after the move

The current local machine cannot run that gate because only one display is
available.

## Local Environment Evidence

Command:

```text
system_profiler SPDisplaysDataType | rg -n "Displays:|Resolution:|UI Looks like|Main Display|Online"
```

Relevant output:

```text
Graphics/Displays:
  Displays:
      Resolution: 3440 x 1440 (UWQHD - Ultra-Wide Quad HD)
      UI Looks like: 3440 x 1440 @ 180.00Hz
      Main Display: Yes
      Online: Yes
```

## Verdict

Multi-display native glass validation remains environment-blocked. The current
single-display geometry probes can validate resize/reposition frame refresh, but
they cannot validate moving between displays or backing-scale changes across
displays.
