# Step 176 - System Screenshot Validation Blocker

Date: 2026-05-10

## Context

Steps 172-175 narrowed the current macOS `AppleNativeInterleave` prototype:

- the AppKit hierarchy can be lower Metal sibling -> native glass -> primary
  Metal view
- the lower scene pass draws
- Studio screenshots now capture the primary foreground instead of the lower
  scene
- the primary foreground screenshot is mostly alpha=0

The remaining question is native AppKit material visibility. Studio framebuffer
screenshots cannot prove that because they do not include the AppKit native
glass layer.

## Attempted Evidence

Command:

```text
osascript -e 'tell application "System Events" to set frontmost of process "makepad-example-aichat" to true'
screencapture -x /tmp/aichat-interleave-system.png
sips -g pixelWidth -g pixelHeight /tmp/aichat-interleave-system.png
```

Output:

```text
/private/tmp/aichat-interleave-system.png
  pixelWidth: 3440
  pixelHeight: 1440
```

The captured image was a black frame and did not show usable desktop or aichat
window composition. It cannot be used as evidence for or against native Liquid
Glass visibility.

## Verdict

The current local automation cannot close the native AppKit material visual
gate. The next required evidence is one of:

- user manual verdict on the currently running
  `makepad-example-aichat-macos-native-clear-interleave-transparent-overlay`
  window
- a usable macOS system-composition screenshot that includes the aichat window
  and desktop content behind it

Until that evidence exists, production `AppleNativeInterleave` remains
unproven.
