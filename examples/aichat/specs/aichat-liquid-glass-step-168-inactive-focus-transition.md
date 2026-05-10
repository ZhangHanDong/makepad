# Step 168 - Native Inactive Focus Transition

Date: 2026-05-10

## Context

Step 135 added macOS app-level activation events, but the earlier runtime run
did not produce `active=false` evidence because automation did not actually
move focus away from the Studio-launched app.

After Step 165 confirmed that system automation can make a Studio-launched
aichat process frontmost, the inactive probe was re-run with an explicit
frontmost-to-Finder transition.

## Runtime Evidence

Studio remote release run:

```text
RunItem makepad-example-aichat-macos-native-clear-inactive-probe
build_id=[21]
```

The run reached native clear State 4:

```text
[liquid-glass] state=4 substrate=macos-native style=clear style_raw=1
[liquid-glass] backend=apple-native-underlay state=Installed containers=1 panels_installed=4 panels_failed=0
```

The initial active state logged:

```text
[liquid-glass] native-inactive-probe active=true style=clear multiplier=1.000
```

Automation then explicitly made `makepad-example-aichat` frontmost, activated
Finder, and confirmed Finder was frontmost:

```text
osascript -e 'tell application "System Events" to set frontmost of process "makepad-example-aichat" to true'
osascript -e 'tell application "Finder" to activate'
osascript -e 'tell application "System Events" to get name of first process whose frontmost is true'

Finder
```

The app then logged the inactive native multiplier:

```text
[liquid-glass] native-inactive-probe active=false style=clear multiplier=0.880
```

## Verdict

The macOS native inactive event source and app-side inactive multiplier are now
runtime-proven for the native clear path.

Remaining scope:

- manual visual assessment on bright/dark wallpapers
- whether Apple-native inactive material behavior should rely more on system
  styling, app-side foreground tokens, or both
